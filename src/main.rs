use std::{
    error::Error,
    io::*,
    cell::RefCell,
    mem,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    ExecutableCommand, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

use openweathermap::{init, update, Receiver};

//Most examples show an overall App struct to organize widgets
struct App<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            titles: vec!["Tab0", "Tab1", "Tab2", "Tab3"], //app struct holds an array for titles to different tabs
            index: 0, //app struct holds an index variable to keep track of where the user is in the array
        }
    }

    //App has a function next that increases by 1, modulus by length to have it wrap around
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) { //similar function, if else for the same wrapping effect
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?; //lines 45-50 are essential for ratatui to run
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let weather_stream = &init("Berlin, DE", "imperial", "en", "f1e875bd567884ff618ff3c7bb8d6e19", 10);
    let weather_string = RefCell::new(String::new());

    let app = App::new(); //creation of the app
    let _res = run_app(&mut terminal, app, weather_stream, &weather_string);

    //lines 58-64 are essential for ratatui to run
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

//run app function draws and handles key events
fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App, stream: &Receiver, string: &RefCell<String>) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app, stream, string))?; //run_app draws the terminal based on a ui function

        if let Event::Key(key) = event::read()? { 
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Right => app.next(),
                    KeyCode::Left => app.previous(),
                    _ => {}
                }
            }
        }
    }
}

//the ui updates the weather information and loads the tabs from the app struct
fn ui(frame: &mut Frame, app: &App, stream: &Receiver, weather_string: &RefCell<String>) {

    let mut weather_string_temp = weather_string.borrow_mut();
    
    match update(stream){
        Some(response) => match response {
            Ok(current) => {
            weather_string_temp.clear();
            weather_string_temp.push_str(&format!("Today's weather in {} is {} and clouds are at {} percent",
            current.name.as_str(),
            current.weather[0].main.as_str(), //addition of random data documentation of the properties this object has-> https://docs.rs/openweathermap/latest/openweathermap/struct.CurrentWeather.html
            current.clouds.all));
        },
            Err(e) => {
                weather_string_temp.clear();
                weather_string_temp.push_str(&format!("Could not fetch weather because: {}", e)); //if the update function fails it writes an error instead
            }
    },
        None => (),
    }

    let titles = app //this thing is setting the titles for the tabs
    .titles
    .iter()
    .map(|t| {
        let (first, rest) = t.split_at(1); //highlights the first letter of every title
        Line::from(vec![first.yellow(), rest.white()])
    })
    .collect();

    //creating the tabs widget
    let tabs = Tabs::new(titles).block(Block::default().borders(Borders::ALL).title("Tabs"))
    .select(app.index)
    .style(Style::default().white().on_blue())
    .highlight_style(Style::default().red().on_white())
    .divider(symbols::DOT);

    //creating a layout for the terminal
    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(3), Constraint::Min(0)])
    .split(frame.size());

    //adding the tabs to the frame in reference to the layout... Its kind of like how you have to set  up
    //a grid pane in JavaFX and then add elements to the grid pane
    frame.render_widget(tabs, chunks[0]);


    //this is styling some boxes. Not super important stuff
    let b = Block::default()
.title(block::Title::from("-Y-O-U-R-").alignment(Alignment::Left))
.title(block::Title::from("-W-E-A-T-H-E-R-").alignment(Alignment::Center))
.title(block::Title::from("-R-E-P-O-R-T-").alignment(Alignment::Right))
.border_type(BorderType::Rounded)
.borders(Borders::ALL);

//depending on the value of index, this changes what is displayed, notice you can define a block and then add it to a paragraph
//I'd like to add a graph to this, I don't know how difficult that will be

    let inner = match app.index {
        0 => Paragraph::new(weather_string_temp.clone()).block(b),
        1 => Paragraph::new("hello").block(Block::default().title("inner 1").borders(Borders::ALL)),
        2 => Paragraph::new("hello once").block(b),
        3 => Paragraph::new("hello again").block(Block::default().title("inner 3").borders(Borders::ALL)),
        _ => unreachable!(),
    };
    frame.render_widget(inner, chunks[1]);
}