use crossterm::cursor::{Hide, Show};
use crossterm::event::{Event, KeyCode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{event, terminal, ExecutableCommand};
use invaders::frame::{new_frame, Drawable};
use invaders::player::Player;
use invaders::render;
use rusty_audio::Audio;
use std::error::Error;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let mut audio = Audio::new();
    audio.add("explode", "explode.wav");
    audio.add("lose", "lose.wav");
    audio.add("move", "move.wav");
    audio.add("pew", "pew.wav");
    audio.add("startup", "startup.wav");
    audio.add("win", "win.wav");
    audio.play("startup");

    // Terminal
    let mut stdout = std::io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;
    // Render loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handler = thread::spawn(move || {
        let mut last_frame = new_frame();
        let mut stdout = std::io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let current_frame = match render_rx.recv() {
                Ok(frame) => frame,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &current_frame, false);
            last_frame = current_frame;
        }
    });
    // Game loop
    let mut player = Player::new();
    let mut instant = Instant::now();
    'game_loop: loop {
        // Per frame init
        let delta = instant.elapsed();
        instant = Instant::now();
        let mut current_frame = new_frame();

        while event::poll(Duration::default())? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if player.shot() {
                            audio.play("pew");
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'game_loop;
                    }
                    _ => {}
                }
            }
        }

        // Updates
        player.update(delta);

        // Draw and render
        player.draw(&mut current_frame);
        let _ = render_tx.send(current_frame);
        thread::sleep(Duration::from_millis(1));
    }

    // Cleanup
    drop(render_tx);
    render_handler.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
