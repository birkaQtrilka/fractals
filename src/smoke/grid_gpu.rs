use ogl33::*;
use crate::compute_ext::{ComputeExt, GL_SHADER_STORAGE_BUFFER, GL_SHADER_STORAGE_BARRIER_BIT};
use crate::learn_opengl::{ComputeProgram};
use crate::smoke::smoke_data::{Pair, Cell};

const INVALID: f32 = -100_000_000.0;

fn make_ssbo(byte_len: usize) -> GLuint {
    let mut id = 0;
    unsafe {
        glGenBuffers(1, &mut id);
        glBindBuffer(GL_SHADER_STORAGE_BUFFER, id);
        glBufferData(GL_SHADER_STORAGE_BUFFER, byte_len as isize, std::ptr::null(), GL_DYNAMIC_DRAW);
    }
    id
}

pub struct GridGpu {
    pub width: usize,
    pub height: usize,
    pub density: f32,
    pub time_step: f32,
    pub cell_size: f32,

    pub smoke: Vec<f32>,
    pub velocities: Vec<Pair>,
    pub solid_map: Vec<bool>,

    velocities_dirty: bool,
    smoke_dirty: bool,

    solid_map_ssbo: GLuint,
    pressures_ssbo: GLuint,
    rhs_ssbo: GLuint,
    inv_total_ssbo: GLuint,
    velocities_ssbo: GLuint,
    temp_velocities_ssbo: GLuint,
    smoke_ssbo: GLuint,
    temp_smoke_ssbo: GLuint,

    prepare_prog: ComputeProgram,
    solve_prog: ComputeProgram,
    update_vel_prog: ComputeProgram,
    advect_vel_prog: ComputeProgram,
    advect_smoke_prog: ComputeProgram,

    group_x: u32,
    group_y: u32,
}

impl GridGpu {
    pub fn new(width: usize, height: usize, density: f32, time_step: f32, cell_size: f32) -> Self {
        let size = width * height;
        let vel_size = (width + 1) * (height + 1);

        let mut velocities = vec![Pair::new(0.0, 0.0); vel_size];
        let smoke = vec![0.0_f32; size];
        let mut solid_map = vec![false; size];

        // Init Boundary Condition
        for i in 0..size {
            let x = i % width;
            let y = i / width;
            let vi = i + y; // x + y * (width + 1)
            
            if x == width - 1 { velocities[vi + 1] = Pair::new(INVALID, 0.0); }
            if y == height - 1 { velocities[vi + width + 1] = Pair::new(0.0, INVALID); }
        }
        velocities[size + height + width] = Pair::new(INVALID, INVALID);

        // Precompile compute shaders
        let prepare_prog = ComputeProgram::from_source(include_str!("../../assets/shaders/prepare_cycle_data.comp")).unwrap();
        let solve_prog = ComputeProgram::from_source(include_str!("../../assets/shaders/red_black_pressure.comp")).unwrap();
        let update_vel_prog = ComputeProgram::from_source(include_str!("../../assets/shaders/update_velocities.comp")).unwrap();
        let advect_vel_prog = ComputeProgram::from_source(include_str!("../../assets/shaders/advect_velocities.comp")).unwrap();
        let advect_smoke_prog = ComputeProgram::from_source(include_str!("../../assets/shaders/advect_smoke.comp")).unwrap();

        let mut grid = GridGpu {
            width, height, density, time_step, cell_size,
            smoke, velocities, solid_map,
            velocities_dirty: true, smoke_dirty: true,
            
            solid_map_ssbo: make_ssbo(size * 4),
            pressures_ssbo: make_ssbo(size * 4),
            rhs_ssbo: make_ssbo(size * 4),
            inv_total_ssbo: make_ssbo(size * 4),
            velocities_ssbo: make_ssbo(vel_size * 8),      // vec2 per cell
            temp_velocities_ssbo: make_ssbo(vel_size * 8), 
            smoke_ssbo: make_ssbo(size * 4),
            temp_smoke_ssbo: make_ssbo(size * 4),
            
            prepare_prog, solve_prog, update_vel_prog, advect_vel_prog, advect_smoke_prog,
            
            group_x: (width as u32 + 7) / 8,
            group_y: (height as u32 + 7) / 8,
        };

        grid.init_solid_map();
        grid
    }

    fn init_solid_map(&mut self) {
        for i in 0..self.width {
            self.solid_map[i] = true;
            self.solid_map[i + self.width * (self.height - 1)] = true;
        }
        for i in (0..(self.width * self.height)).step_by(self.width) {
            self.solid_map[i] = true;
            self.solid_map[i + self.width - 1] = true;
        }
        self.upload_solid_map();
    }

    // Call this explicitly if you manually edit the CPU-side `solid_map`!
    pub fn upload_solid_map(&self) {
        let as_u32: Vec<u32> = self.solid_map.iter().map(|&b| b as u32).collect();
        Self::upload(self.solid_map_ssbo, &as_u32);
    }

    pub fn set_velocities(&mut self, pressure_index: usize, t: Option<f32>, l: Option<f32>, b: Option<f32>, r: Option<f32>) {
        let y = pressure_index / self.width;
        let vi = pressure_index + y;

        let vel = self.velocities[vi];
        self.velocities[vi] = Pair::new(t.unwrap_or(vel.top), l.unwrap_or(vel.left));

        let vel = self.velocities[vi + 1];
        self.velocities[vi + 1] = Pair::new(vel.top, r.unwrap_or(vel.left));

        let vel = self.velocities[vi + self.width + 1];
        self.velocities[vi + self.width + 1] = Pair::new(b.unwrap_or(vel.top), vel.left);
        
        self.velocities_dirty = true;
    }

    pub fn get_velocities(&self, pressure_index: usize) -> Cell {
        let y = pressure_index / self.width;
        let vi = pressure_index + y;
        let t = self.velocities[vi].top;
        let l = self.velocities[vi].left;
        let b = self.velocities[vi + self.width + 1].top;
        let r = self.velocities[vi + 1].left;
        Cell::new(t, l, b, r)
    }

    pub fn set_smoke(&mut self, i: usize, val: f32) {
        self.smoke[i] = val;
        self.smoke_dirty = true;
    }

    fn upload<T>(ssbo: GLuint, data: &[T]) {
        unsafe {
            glBindBuffer(GL_SHADER_STORAGE_BUFFER, ssbo);
            glBufferSubData(
                GL_SHADER_STORAGE_BUFFER, 0,
                (data.len() * std::mem::size_of::<T>()) as isize,
                data.as_ptr().cast(),
            );
        }
    }

    fn download<T>(ssbo: GLuint, data: &mut [T]) {
        unsafe {
            glBindBuffer(GL_SHADER_STORAGE_BUFFER, ssbo);
            glGetBufferSubData(
                GL_SHADER_STORAGE_BUFFER, 0,
                (data.len() * std::mem::size_of::<T>()) as isize,
                data.as_mut_ptr().cast(),
            );
        }
    }

    pub fn step(&mut self, ext: &ComputeExt, pressure_iterations: u32) {
        if self.velocities_dirty {
            Self::upload(self.velocities_ssbo, &self.velocities);
            self.velocities_dirty = false;
        }
        if self.smoke_dirty {
            Self::upload(self.smoke_ssbo, &self.smoke);
            self.smoke_dirty = false;
        }

        unsafe {
            // Bind all shared static SSBOs initially
            glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 0, self.solid_map_ssbo);
            glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 1, self.pressures_ssbo);
            glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 2, self.rhs_ssbo);
            glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 3, self.inv_total_ssbo);
            glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 4, self.velocities_ssbo);

            // 1. Prepare Cycle Data
            glUseProgram(self.prepare_prog.0);
            let ady = self.density * self.cell_size / self.time_step;
            glUniform1i(self.prepare_prog.get_unif_location("width"), self.width as i32);
            glUniform1i(self.prepare_prog.get_unif_location("height"), self.height as i32);
            glUniform1f(self.prepare_prog.get_unif_location("ady"), ady);
            (ext.dispatch_compute)(self.group_x, self.group_y, 1);
            (ext.memory_barrier)(GL_SHADER_STORAGE_BARRIER_BIT);

            // 2. Solve Pressure (Red-Black iterations)
            glUseProgram(self.solve_prog.0);
            glUniform1i(self.solve_prog.get_unif_location("width"), self.width as i32);
            glUniform1i(self.solve_prog.get_unif_location("height"), self.height as i32);
            let parity_loc = self.solve_prog.get_unif_location("parity");
            for _ in 0..pressure_iterations {
                for parity in 0..=1 {
                    glUniform1i(parity_loc, parity);
                    (ext.dispatch_compute)(self.group_x, self.group_y, 1);
                    (ext.memory_barrier)(GL_SHADER_STORAGE_BARRIER_BIT);
                }
            }

            // 3. Update Velocities
            glUseProgram(self.update_vel_prog.0);
            let k = self.time_step / (self.cell_size * self.density);
            glUniform1i(self.update_vel_prog.get_unif_location("width"), self.width as i32);
            glUniform1i(self.update_vel_prog.get_unif_location("height"), self.height as i32);
            glUniform1f(self.update_vel_prog.get_unif_location("k"), k);
            (ext.dispatch_compute)(self.group_x, self.group_y, 1);
            (ext.memory_barrier)(GL_SHADER_STORAGE_BARRIER_BIT);

            // 4. Advect Velocities
            glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 5, self.temp_velocities_ssbo);
            glUseProgram(self.advect_vel_prog.0);
            glUniform1i(self.advect_vel_prog.get_unif_location("width"), self.width as i32);
            glUniform1i(self.advect_vel_prog.get_unif_location("height"), self.height as i32);
            glUniform1f(self.advect_vel_prog.get_unif_location("cell_size"), self.cell_size);
            glUniform1f(self.advect_vel_prog.get_unif_location("time_step"), self.time_step);
            (ext.dispatch_compute)(self.group_x, self.group_y, 1);
            (ext.memory_barrier)(GL_SHADER_STORAGE_BARRIER_BIT);
            
            // Ping-pong buffer binding references internally to avoid copying memory!
            std::mem::swap(&mut self.velocities_ssbo, &mut self.temp_velocities_ssbo);

            // 5. Advect Smoke
            glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 4, self.velocities_ssbo); // Rebind freshly updated velocities
            glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 6, self.smoke_ssbo);
            glBindBufferBase(GL_SHADER_STORAGE_BUFFER, 7, self.temp_smoke_ssbo);
            glUseProgram(self.advect_smoke_prog.0);
            glUniform1i(self.advect_smoke_prog.get_unif_location("width"), self.width as i32);
            glUniform1i(self.advect_smoke_prog.get_unif_location("height"), self.height as i32);
            glUniform1f(self.advect_smoke_prog.get_unif_location("cell_size"), self.cell_size);
            glUniform1f(self.advect_smoke_prog.get_unif_location("time_step"), self.time_step);
            (ext.dispatch_compute)(self.group_x, self.group_y, 1);
            (ext.memory_barrier)(GL_SHADER_STORAGE_BARRIER_BIT);

            std::mem::swap(&mut self.smoke_ssbo, &mut self.temp_smoke_ssbo);
        }

        // Pull results back down for the Interactor/Drawer
        Self::download(self.velocities_ssbo, &mut self.velocities);
        Self::download(self.smoke_ssbo, &mut self.smoke);
    }
}