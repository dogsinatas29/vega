use std::time::Duration;
use crate::context::SystemContext;
use super::renderer::Renderer;
use super::mapper::SystemMapper;

pub struct DoomEngine {
    renderer: Renderer,
    mapper: SystemMapper,
    is_running: bool,
}

impl DoomEngine {
    pub fn new() -> Self {
        Self {
            renderer: Renderer::new(100, 40), // Target 100x40 resolution
            mapper: SystemMapper::new(),
            is_running: false,
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        use crossterm::{
            event::{self, KeyCode, Event},
            terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
            ExecutableCommand,
        };
        use std::io::stdout;

        let mut stdout = stdout();
        stdout.execute(EnterAlternateScreen)?;
        enable_raw_mode()?;

        self.is_running = true;

        // Game Loop
        while self.is_running {
            // 1. Input
            if event::poll(Duration::from_millis(10))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => self.is_running = false,
                        KeyCode::Char('w') => self.renderer.move_camera(0.1),
                        KeyCode::Char('s') => self.renderer.move_camera(-0.1),
                        KeyCode::Char('a') => self.renderer.rotate_camera(-0.1),
                        KeyCode::Char('d') => self.renderer.rotate_camera(0.1),
                        _ => {}
                    }
                }
            }

            // 2. Update System Data
            let ctx = SystemContext::collect();
            let map_data = self.mapper.update(&ctx);

            // 3. Render
            self.renderer.render(&map_data, &mut stdout)?;

            // Cap FPS
            std::thread::sleep(Duration::from_millis(30)); // ~30 FPS
        }

        disable_raw_mode()?;
        stdout.execute(LeaveAlternateScreen)?;
        Ok(())
    }
}
