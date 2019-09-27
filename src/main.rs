extern crate serde_derive;
extern crate rand;
use termcolor::{Color, Ansi, ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::io::Write;
use std::option;
use rand::Rng;
use std::net::{UdpSocket, SocketAddr, IpAddr, Ipv4Addr};
use std::{thread, time};
use std::vec::Vec;
pub mod device_modules;
mod audio;
use audio::{init_audio, init_audio_simple, Vec2, Vec4, SAMPLE_RATE};

use device_modules::config::*;
use std::io;
use std::sync::mpsc::*;

mod visualizer;
use visualizer::display;

use std::f64;
use std::f32;


// mod plotting;
// use plotting::*;

#[allow(unreachable_code)]
fn setup_device(cfg_settings:&mut Devicecfg) -> (StatusType) {
    let mut input = String::new();
    let mut cfg_complete = false;

    println!("Setup custom config: s");
    println!("Use Default settings: d");
    println!("Quit: q");
    while !cfg_complete {
        match io::stdin().read_line(&mut input) {
            Ok(_number_of_bytes) => {
                match input.trim() {
                    "s" => {unimplemented!()},
                    "d" => {
                        cfg_complete = true;
                        *cfg_settings =  Devicecfg::default();
                    }
                    _  => { 
                        panic!("Unhandled case")
                    },
                    
                };
            }
            Err(error) => println!("error: {}", error),
        }
    }
    return StatusType::ERROR;
}

fn config_mode() {
    let mut device_settings = Devicecfg::default();
    setup_device(& mut device_settings);
    println!("Setup Remote Device ? y/n");
    //TODO make this a macro
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_number_of_bytes) => {
            match input.trim() {
                "y" => {unimplemented!()},
                "n" => {
                },
                _  => { 
                    panic!("Unhandled case")
                },
                
            };
        }
        Err(error) => println!("error: {}", error),
    }

}

fn make_random_led_vec(strip_size : usize) -> Vec<Vec<u8>> {
        let mut test_leds : Vec<Vec<u8>> = Vec::with_capacity(strip_size);
        let mut rng = rand::thread_rng();

        for _ in 0..strip_size {
            let led_idx: Vec<u8> = (0..3).map(|_| {
                rng.gen_range(0,255)
                }).collect();
            test_leds.push(led_idx);
        }
        test_leds
}

struct Pixel {
    //Named with a U because 🇨🇦
    pub colour : [u8;3],
    pub stdout_symbol : String,
    colour_spec : ColorSpec,
}

// impl Pixel {
    // fn new(setup_colour : [u8;3], symbol : Option<String>) -> Pixel {
        // let mut symbol_defined = false;
        // if let Some(x) = symbol {
            // symbol_defined = true;
        // }
                // let std_out = "symbol";
        // match symbol_defined {
            // true => {
                // let std_out = "symbol";
            // }   
            // _ => {
                // let std_out = " ";
            // }
        // }
        // Pixel { 
            // colour : setup_colour,
            // stdout_symbol : std_out,
            // colour_spec : ColorSpec::new().set_fg(Some(Colour::Rgb(setup_colour)))
        // }
    // }
// }

//Dont know how to return a mutable result which contains a vec
// fn get_frequencies(resolution : f64) -> std::io::Result<Vec<64>> {
//     let mut freq_vec : Vec<f64> = Vec::with_capacity(fft_size);
//     for (bin_idx, _) in (0..fft_size).enumerate(){
//         freq_vec.push(bin_idx * freq_res);
//     }
// }

fn get_freq_chart(audio_buff : &Vec<Vec4>, vec_size : usize, use_polar : bool) -> std::io::Result<(Vec<(f32, f32)>)> {
    let mut freq_mag : Vec<(f32,f32)> = Vec::with_capacity(vec_size);
    for audio_packet in audio_buff.iter() {
        let real_part = audio_packet.vec[0];
        let im_part = audio_packet.vec[1];
        //Unused for now
        let freq = audio_packet.vec[2];
        // let ang_velocity = audio_packet.vec[2];
        // let ang_noise = audio_packet.vec[3];
        if use_polar {
            let mag_polar = f32::sqrt(real_part.exp2() + im_part.exp2());
            let mag_db_polar = 20.0f32*(2.0f32*mag_polar/vec_size as f32).abs().log10();
            freq_mag.push((freq, mag_db_polar));
        } else {
            let mag_db_rect = 20.0f32*(((2.0f32*im_part/vec_size as f32).abs()).log10());
            freq_mag.push((freq, mag_db_rect));
        }
        // println!("mag: {:?}", 20.0f32*(mag.log10()));
    }
    Ok(freq_mag)
}

fn main() -> std::io::Result<()> {
    {
        let esp_if = Devicecfg::default();
        // let esp_addr = SocketAddr::new(esp_if.device_specific_cfg.udp_ip, esp_if.device_specific_cfg.udp_port);
        let esp_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5005);
        let init_strip = make_random_led_vec(25);
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        // writeln!(&mut stdout, "green text!");
        let mut box_buff : termcolor::Buffer;// = "█";
        // for led_idx in init_strip {
        //     // let pixel = Pixel::new(led_idx);
        //     stdout.set_color(ColorSpec::new().set_fg(Some(Color::Rgb(led_idx[0], led_idx[1], led_idx[2]))));
        //     println!("▀");
        //     // println!("led_val: {:?}", led_idx);
        //     update_esp8266(esp_addr, &led_idx)?;
        // }
        
        //get the frequency portion of the frequencyxmagnitude graph
        let fft_size : usize = 1024;

        let freq_res = SAMPLE_RATE as f32/fft_size as f32; //frequency resolution
        let mut freq_vec : Vec<f32> = Vec::with_capacity(fft_size);
        for (bin_idx, _) in (0..fft_size).enumerate(){
            freq_vec.push(bin_idx as f32 * freq_res);
        }
        //Start the audio stream
        let (mut stream, buffers) = init_audio_simple(&esp_if).unwrap();
        // let (mut stream, buffers) = init_audio(&esp_if).unwrap();

        stream.start().expect("Unable the open stream");
        thread::sleep(time::Duration::from_secs(5));
        let handle = thread::spawn(move || {
            let mut index = 0;
            // for _ in (0..1024-259) {
                while !buffers[index].lock().unwrap().rendered {
                    let mut buffer = buffers[index].lock().unwrap();
                    //This is 258 since it needs to store the full range + 3 values to maintain
                    //Continuity
                    // ys_data.copy_from_slice(&buffer.analytic);
                    buffer.rendered = true;
                    index = (index + 1) % buffers.len();
                    //here we borrow a reference to buffer.analytic 
                    //this allows get_freq_chart to use the data but ensure nothing else 
                    //can manipulate it
                    // println!("size is {:?}",buffer.analytic.len());
                    //make sure to unwrap Results to properly iterate
                    let freq_mag = get_freq_chart(&buffer.analytic, fft_size, false).unwrap();
                    // println!("From buffer");
                    for val in freq_mag.iter() {
                        println!("{:?}, {:?}", val.0, val.1);
                    }
                }
            // }
        });
        // display(buffers);
        handle.join().unwrap();   
        // stream.stop();

    }
    Ok(()) 
}


fn colour_from_vert4(base_hue : f32, decay : f32, desaturation : f32, relative_length : f32, angle : Vec2, position : f32) -> std::io::Result<(f64)> {
    let colour : f64 = 0.0;

    Ok(colour)
}


fn update_esp8266(socket_address : SocketAddr, esp_packet : &[u8]) -> std::io::Result<()> {
    /*
    The ESP8266 will receive and decode the packets to determine what values
    to display on the LED strip. The communication protocol supports LED strips
    with a maximum of 256 LEDs.

        |i|r|g|b|
    where
        i (0 to 255): Index of LED to change (zero-based)
        r (0 to 255): red value of LED
    The packet encoding scheme is:
        g (0 to 255): green value of LED
        b (0 to 255): blue value of LED
    */
    {
        let local_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 5010);
        let socket = UdpSocket::bind(local_address)?;
        socket.send_to(esp_packet, socket_address)?;
    }
    Ok(())
}

extern crate itertools;
use itertools::Itertools;

// fn split_audio_spectrum(audio_spectrum : &Vec<f32>) -> std::io::Result<([f32;3])> {
    
// }

/*
    From Wavelength to RGB in Python - https://www.noah.org/wiki/Wavelength_to_RGB_in_Python
    == A few notes about color ==

    Color   Wavelength(nm) Frequency(THz)
    red     620-750        484-400
    Orange  590-620        508-484
    Yellow  570-590        526-508
    green   495-570        606-526
    blue    450-495        668-606
    Violet  380-450        789-668

    f is frequency (cycles per second)
    l (lambda) is wavelength (meters per cycle)
    e is energy (Joules)
    h (Plank's constant) = 6.6260695729 x 10^-34 Joule*seconds
                         = 6.6260695729 x 10^-34 m^2*kg/seconds
    c = 299792458 meters per second
    f = c/l
    l = c/f
    e = h*f
    e = c*h/l

    List of peak frequency responses for each type of 
    photoreceptor cell in the human eye:
        S cone: 437 nm
        M cone: 533 nm
        L cone: 564 nm
        rod:    550 nm in bright daylight, 498 nm when dark adapted. 
                Rods adapt to low light conditions by becoming more sensitive.
                Peak frequency response shifts to 498 nm.
*/

const MINIMUM_VISIBLE_WAVELENGTH :u16 = 380;
const MAXIMUM_VISIBLE_WAVELENGTH :u16 = 740;

fn wavelength_to_rgb(wavelength : f32, gamma : f32) -> std::io::Result<([u8;3])> {
    let red : f32;
    let green : f32;
    let blue : f32;

    if wavelength > 440.0 && wavelength < 490.0 {
        let attenuation = 0.3 + 0.7*(wavelength 
            - MINIMUM_VISIBLE_WAVELENGTH as f32);
        red = (-wavelength - 440.0) / (440.0 - MINIMUM_VISIBLE_WAVELENGTH as f32)
             * attenuation * gamma;
        green = 0.0;
        blue = 1.0 * attenuation;
    }
    else if wavelength >= 440.0 && wavelength <= 490.0 {
        red = 0.0;
        green = ((wavelength - 440.0) / (490.0 - 440.0)) * gamma;
        blue = 1.0;
    }
    else if wavelength >= 490.0 && wavelength <= 510.0 {
        red = 0.0;
        green = 1.0;
        blue = (-(wavelength - 510.0) / (510.0 - 490.0)) * gamma;
    }
    else if wavelength >= 510.0 && wavelength <= 580.0 {
        red = ((wavelength - 510.0) / (580.0 - 510.0)) * gamma;
        green = 1.0;
        blue = 0.0;
    }
    else if wavelength >= 580.0 && wavelength <= 645.0 {
        red = 1.0;
        green = (-(wavelength - 645.0) / (645.0 - 580.0)) * gamma;
        blue = 0.0;
    }
    else if wavelength >= 645.0 && wavelength 
        <= MAXIMUM_VISIBLE_WAVELENGTH as f32 {
        let attenuation = 0.3 + 0.7 * 
            (MAXIMUM_VISIBLE_WAVELENGTH as f32 - wavelength) 
            / (MAXIMUM_VISIBLE_WAVELENGTH as f32 - 645.0);
        red = (1.0 * attenuation) * gamma;
        green = 0.0;
        blue = 0.0;
    }
    else {
        red = 0.0;
        green = 0.0;
        blue = 0.0;
    }
    let rgb = [(255.0 * red) as u8, (255.0 * green) as u8, (255.0 * blue) as u8];

    Ok(rgb)
}

fn map_synthesia(audio_range : [f32; 2], audio_value : f32) -> std::io::Result<(f32)> {
//affline transform
//for now audio_range[0] is min and audio_range[1] is max
    let res = (audio_value - audio_range[0]) 
        * ((MAXIMUM_VISIBLE_WAVELENGTH-MINIMUM_VISIBLE_WAVELENGTH) as f32)
        /(audio_range[1] - audio_range[0]) + MINIMUM_VISIBLE_WAVELENGTH as f32;
    Ok(res)
}
// Receives a single datagram message on the socket. If `buf` is too small to hold
// the message, it will be cut off.
// let mut buf = [0; 10];
// let (amt, src) = socket.recv_from(&mut buf)?;
// println!("src is {:?}", src);

// redeclare `buf` as slice of the received data and send reverse data back to origin.
// let buf = &mut buf[..amt];
// buf.reverse();
// println!("buf is {:?} src is {:?}", buf, src);