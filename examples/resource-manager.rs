extern crate sdl2;

use sdl2::event::Event;
use sdl2::image::{LoadTexture, InitFlag};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{TextureCreator, Texture};
use sdl2::ttf::{Font, Sdl2TtfContext};

use std::env;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

fn main() {
    let args: Vec<_> = env::args().collect();

    if args.len() < 3 {
        println!("Usage: cargo run /path/to/image.(png|jpg) /path/to/font.(ttf|ttc|fon)")
    } else {
        let image_path = &args[1];
        let font_path = &args[2];

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let font_context = sdl2::ttf::init().unwrap();
        let _image_context = sdl2::image::init(InitFlag::INIT_PNG | InitFlag::INIT_JPG).unwrap();
        let window = video_subsystem
            .window("rust-sdl2 resource-manager demo", 800, 600)
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().software().build().unwrap();
        let texture_creator = canvas.texture_creator();
        let mut texture_manager = TextureManager::new(&texture_creator);
        let mut font_manager = FontManager::new(&font_context);
        let details = FontDetails {
            path: font_path.clone(),
            size: 32,
        };

        'mainloop: loop {
            for event in sdl_context.event_pump().unwrap().poll_iter() {
                match event {
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } |
                    Event::Quit { .. } => break 'mainloop,
                    _ => {}
                }
            }
            // will load the image texture + font only once
            let texture = texture_manager.load(image_path).unwrap();
            let font = font_manager.load(&details).unwrap();

            // not recommended to create a texture from the font each iteration
            // but it is the simplest thing to do for this example
            let surface = font.render("Hello Rust!")
                .blended(Color::RGBA(255, 0, 0, 255))
                .unwrap();
            let font_texture = texture_creator
                .create_texture_from_surface(&surface)
                .unwrap();

            //draw all
            canvas.clear();
            canvas.copy(&texture, None, None).unwrap();
            canvas.copy(&font_texture, None, None).unwrap();
            canvas.present();
        }
    }
}

type TextureManager<'l, T> = ResourceManager<'l, String, Texture<'l>, TextureCreator<T>>;
type FontManager<'l> = ResourceManager<'l, FontDetails, Font<'l, 'static>, Sdl2TtfContext>;

// Generic struct to cache any resource loaded by a ResourceLoader
pub struct ResourceManager<'l, K, R, L>
    where K: Hash + Eq,
          L: 'l + ResourceLoader<'l, R>
{
    loader: &'l L,
    cache: HashMap<K, Rc<R>>,
}

impl<'l, K, R, L> ResourceManager<'l, K, R, L>
    where K: Hash + Eq,
          L: ResourceLoader<'l, R>
{
    pub fn new(loader: &'l L) -> Self {
        ResourceManager {
            cache: HashMap::new(),
            loader: loader,
        }
    }

    // Generics magic to allow a HashMap to use String as a key
    // while allowing it to use &str for gets
    pub fn load<D>(&mut self, details: &D) -> Result<Rc<R>, String>
        where L: ResourceLoader<'l, R, Args = D>,
              D: Eq + Hash + ?Sized,
              K: Borrow<D> + for<'a> From<&'a D>
    {
        self.cache
            .get(details)
            .cloned()
            .map_or_else(|| {
                             let resource = Rc::new(self.loader.load(details)?);
                             self.cache.insert(details.into(), resource.clone());
                             Ok(resource)
                         },
                         Ok)
    }
}

// TextureCreator knows how to load Textures
impl<'l, T> ResourceLoader<'l, Texture<'l>> for TextureCreator<T> {
    type Args = str;
    fn load(&'l self, path: &str) -> Result<Texture, String> {
        println!("LOADED A TEXTURE");
        self.load_texture(path)
    }
}

// Font Context knows how to load Fonts
impl<'l> ResourceLoader<'l, Font<'l, 'static>> for Sdl2TtfContext {
    type Args = FontDetails;
    fn load(&'l self, details: &FontDetails) -> Result<Font<'l, 'static>, String> {
        println!("LOADED A FONT");
        self.load_font(&details.path, details.size)
    }
}

// Generic trait to Load any Resource Kind
pub trait ResourceLoader<'l, R> {
    type Args: ?Sized;
    fn load(&'l self, data: &Self::Args) -> Result<R, String>;
}

// Information needed to load a Font
#[derive(PartialEq, Eq, Hash)]
pub struct FontDetails {
    pub path: String,
    pub size: u16,
}

impl<'a> From<&'a FontDetails> for FontDetails {
    fn from(details: &'a FontDetails) -> FontDetails {
        FontDetails {
            path: details.path.clone(),
            size: details.size,
        }
    }
}
