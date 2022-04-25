use gloo_console::log;
use gloo_utils::*;
use web_sys::HtmlElement;
use yew::prelude::*;

mod game;

use game::*;

enum Msg {
    /// Making the board with the field indicating the
    MakeBoard { size: usize },
    /// A click on the go board, fields are client x
    /// and y values of the click
    Click { x: i32, y: i32 },
    /// Signals that the window or something has been resized
    Resize,
}

struct App {
    board_ref: NodeRef,
    board: GoGame,
    preview: Option<(usize, usize)>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        App {
            board_ref: NodeRef::default(),
            board: GoGame::new(0),
            preview: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            // TODO: implement creations for every size
            // currently must be odd so dots work properly
            Msg::MakeBoard { size: x } if [5, 7, 9, 13].contains(&x) => {
                self.board = GoGame::new(x);
                true
            }
            Msg::MakeBoard { .. } => {
                self.board = GoGame::default();
                true
            }
            Msg::Click { x, y } => {
                let border_width = self.get_tile_border_width() as f64;
                let tile_size = self.get_tile_size() as f64;
                let board_padding = self.get_board_padding() as f64;

                let x = x as usize - (board_padding - tile_size / 2.0) as usize;
                let y = y as usize - (board_padding - tile_size / 2.0) as usize;
                let end_limit = tile_size as usize * self.board.board_size()
                    + border_width as usize * self.board.board_size();
                if !(0..end_limit).contains(&x) || !(0..end_limit).contains(&y) {
                    return false;
                }

                let x = x / (tile_size + border_width) as usize;
                let y = y / (tile_size + border_width) as usize;

                match self.preview {
                    Some(preview_coords) if preview_coords == (x, y) => {
                        self.preview = None;
                        // Play the move on the board
                        self.board.play_move(x, y).is_ok()
                    }
                    _ => {
                        // Check if position can be played on
                        if self.board.is_valid_move(x, y, self.board.next_player) {
                            self.preview = Some((x, y));
                            true
                        } else {
                            self.preview = None;
                            true
                        }
                    }
                }
            }
            Msg::Resize => {
                log!("Resized!");
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match self.board.board_size() {
            0 => {
                let button_onclick = ctx.link().callback(move |_| Msg::MakeBoard { size: 19 });
                html! {
                    <>
                        <button onclick={ button_onclick }>{ "Default" }</button>
                        <table class="g-board" style="display: none;">
                            <td></td>
                        </table>
                    </>
                }
            }
            _ => {
                let board_ref = self.board_ref.clone();
                let board_oncontext = ctx.link().callback(move |e: MouseEvent| {
                    e.prevent_default();
                    let board = board_ref.cast::<HtmlElement>().unwrap();
                    let rect = board.get_bounding_client_rect();
                    let mouse_x = ((e.client_x() as f64) - rect.left()) as i32;
                    let mouse_y = ((e.client_y() as f64) - rect.top()) as i32;
                    Msg::Click {
                        x: mouse_x,
                        y: mouse_y,
                    }
                });
                let board_onresize = ctx.link().callback(move |_: Event| Msg::Resize);
                let board = self.make_board_ref();
                let dots = self.make_dots_html();
                let preview = self.render_preview();
                let tiles = self.render_moves();

                // Return full html
                html! {
                    <div
                        ref={ self.board_ref.clone() }
                        onclick={ board_oncontext }
                        onresize={ board_onresize }
                        class="g-container"
                    >
                        { preview }
                        { dots }
                        { tiles }
                        { board }
                    </div>
                }
            }
        }
    }
}

impl App {
    fn render_preview(&self) -> Html {
        match self.preview {
            None => {
                html! {}
            }
            Some((x, y)) => {
                let tile_size = self.get_tile_size();
                let shift_size = tile_size + self.get_tile_border_width();
                let offset: i32 = tile_size as i32 / 2;
                match self.board.next_player {
                    Player::None => {
                        html! {}
                    }
                    Player::White => {
                        html! {
                            <div
                                class="g-preview-white"
                                style={ format!(
                                    "position: absolute; transform: translate({}.5px, {}.5px)",
                                    (shift_size * x) as i32 - offset,
                                    (shift_size * y) as i32 - offset,
                                )}>
                            </div>
                        }
                    }
                    Player::Black => {
                        html! {
                            <div
                                class="g-preview-black"
                                style={ format!(
                                    "position: absolute; transform: translate({}.5px, {}.5px)",
                                    (shift_size * x) as i32 - offset,
                                    (shift_size * y) as i32 - offset,
                                )}>
                            </div>
                        }
                    }
                }
            }
        }
    }
    /// Renders moves, but it preparational style
    fn render_moves(&self) -> Html {
        const TILE_MODIFIER: f64 = 0.45;

        let board_size = self.board.board_size();
        let board_padding = self.get_board_padding();
        let mut tiles = Vec::with_capacity(board_size);
        for (i, player) in self.board.position().iter().enumerate() {
            let (x, y) = self.board.index_to_coord(i);
            let tile_size = self.get_tile_size();
            let shift_size = tile_size + self.get_tile_border_width();

            let shift_x = shift_size * x + board_padding;
            let shift_y = shift_size * y + board_padding;

            // Get computed style
            let body_style = window().get_computed_style(&body()).unwrap().unwrap();

            match *player {
                Player::None => {}
                Player::White => {
                    let white = body_style.get_property_value("--fg-white").unwrap();
                    let tile = html! {
                        <circle
                            cx={ shift_x.to_string() }
                            cy={ shift_y.to_string() }
                            r={ format!("{:.2}", (tile_size as f64 * TILE_MODIFIER)) }
                            fill={ white }
                        >
                        </circle>
                    };
                    tiles.push(tile);
                }
                Player::Black => {
                    let black = body_style.get_property_value("--fg-black").unwrap();
                    let tile = html! {
                        <circle
                            cx={ shift_x.to_string() }
                            cy={ shift_y.to_string() }
                            r={ format!("{:.2}", (tile_size as f64 * TILE_MODIFIER)) }
                            fill={ black }
                        >
                        </circle>
                    };
                    tiles.push(tile);
                }
            }
        }
        let svg_size = self.get_tile_size() as usize * (self.board.board_size() - 1)
            + self.get_tile_border_width() as usize * self.board.board_size();
        html! {
            <svg width={ (svg_size + 2 * board_padding).to_string() }
                height={ (svg_size + 2 * board_padding).to_string() }
                style={ format!("transform: translate(-{0}px, -{0}px);", board_padding) }
                fill="none" xmlns="http://www.w3.org/2000/svg">
                { for tiles }
            </svg>
        }
    }

    fn get_tile_size(&self) -> usize {
        let tile = document().query_selector(".g-board td").unwrap().unwrap();
        let tile_style = window().get_computed_style(&tile).unwrap().unwrap();
        let tile_size = tile_style
            .get_property_value("width")
            .unwrap()
            // Get rid of the unit on the end, presumably "px"
            .chars()
            .filter(|c| c.is_numeric())
            .collect::<String>()
            // convert to f64
            .parse::<usize>()
            .unwrap();
        tile_size
    }

    /// Assumes there is a <td> element under an element with class
    /// `g-board`. Panics otherwise.
    fn get_tile_border_width(&self) -> usize {
        let tile = document().query_selector(".g-board td").unwrap().unwrap();
        let tile_style = window().get_computed_style(&tile).unwrap().unwrap();
        let border_width = tile_style
            .get_property_value("border-top-width")
            .unwrap()
            // Get rid of the unit on the end, presumably "px"
            .chars()
            .filter(|c| c.is_numeric())
            .collect::<String>()
            // convert to f64
            .parse::<usize>()
            .unwrap();
        border_width
    }

    fn get_board_padding(&self) -> usize {
        let board = document().query_selector(".g-container").unwrap();
        if board.is_none() {
            return 0;
        }
        let board = board.unwrap();
        let board_style = window().get_computed_style(&board).unwrap().unwrap();
        let board_padding = board_style
            .get_property_value("padding-left")
            .unwrap()
            // Get rid of the unit on the end, presumably "px"
            .chars()
            .filter(|c| c.is_numeric())
            .collect::<String>()
            // convert to f64
            .parse::<usize>()
            .unwrap();
        board_padding
    }

    fn make_board_ref(&self) -> Html {
        let mut board = Vec::with_capacity(self.board.board_size() - 1);
        for _ in 0..self.board.board_size() - 1 {
            let mut board_row = Vec::with_capacity(self.board.board_size() - 1);
            for _ in 0..self.board.board_size() - 1 {
                let tile_html = html! {
                    <td></td>
                };
                board_row.push(tile_html);
            }
            let row_html = html! {
                <tr>{ for board_row }</tr>
            };
            board.push(row_html);
        }
        html! {
            <table class="g-board">{ for board }</table>
        }
    }

    fn make_dots_html(&self) -> Html {
        // Retrieve some values from the stylesheet
        let border_width = self.get_tile_border_width() as f64;
        let box_size = self.get_tile_size() as f64;
        println!("{}, {} = (1, 31)?", border_width, box_size);

        let svg_size = box_size as usize * (self.board.board_size() - 1)
            + border_width as usize * self.board.board_size();

        // Make circle svgs
        let coords = [
            self.board.board_size() / 4 - 1,
            self.board.board_size() / 2,
            self.board.board_size() - (self.board.board_size() / 4),
        ];
        let coords_iter = coords
            .into_iter()
            .flat_map(|x| coords.into_iter().map(|y| (x, y)).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let mut circles_svg = Vec::with_capacity(9);
        for (x, y) in coords_iter {
            let x: f64 = 0.5 + (box_size + border_width) * x as f64;
            let y: f64 = 0.5 + (box_size + border_width) * y as f64;
            let r: usize = 3;

            let circle = html! {
                <circle cx={ x.to_string() }
                    cy={ y.to_string() }
                    r={ r.to_string() }
                    fill="black" />
            };
            circles_svg.push(circle);
        }

        html! {
            <svg width={ svg_size.to_string() } height={ svg_size.to_string() } fill="none" xmlns="http://www.w3.org/2000/svg">
                { for circles_svg }
            </svg>
        }
    }
}

fn main() {
    yew::start_app::<App>();
}
