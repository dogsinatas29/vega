use crate::context::SystemContext;

pub struct SystemMapper {
    pub map: Vec<Vec<u8>>,
    tick: u64,
}

impl SystemMapper {
    pub fn new() -> Self {
        let size = 20;
        let mut map = vec![vec![0; size]; size];
        
        // Boundaries
        for i in 0..size {
            map[0][i] = 1;
            map[size-1][i] = 1;
            map[i][0] = 1;
            map[i][size-1] = 1;
        }

        Self {
            map,
            tick: 0,
        }
    }

    pub fn update(&mut self, _ctx: &SystemContext) -> Vec<Vec<u8>> {
        self.tick += 1;
        
        // Animate walls based on "load" (simulated for now if load_avg is static)
        // In real impl, we'd map ctx.load_avg[0] to wall intensity
        
        // Pulsing center pillar
        if self.tick % 10 == 0 {
             self.map[10][10] = if self.map[10][10] == 0 { 2 } else { 0 };
        }
        
        self.map.clone() // Clone for renderer safety (in real usage we'd use RefCell or pass ref)
    }
}
