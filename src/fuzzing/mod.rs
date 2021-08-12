use rand::prelude::*;

pub fn rand_string(rng: &mut ThreadRng) -> String {
    (0..4).map(|_| rng.gen::<char>()).collect()
}

pub fn gen_spaces(rng: &mut ThreadRng) -> String {
    let no_space: bool = rng.gen::<f64>() > 0.05;
    if no_space {
        "".to_string()
    } else {
        (0..rng.gen_range(0..=4)).map(|_|" ").collect::<Vec<&str>>().join("")
    }
    
}

pub fn gen_client(rng: &mut ThreadRng) -> String {
    let correct: bool = rng.gen::<f64>() > 0.01;
    if correct {
        // rng.gen::<u16>().to_string()
        rng.gen_range(0..=1000).to_string()
    } else {
        rand_string(rng)
    }
}

pub fn gen_tx(rng: &mut ThreadRng) -> String {
    let correct: bool = rng.gen::<f64>() > 0.01;
    if correct {
        rng.gen_range(0..=1000).to_string()
        // rng.gen::<u32>().to_string()
    } else {
        rand_string(rng)
    }
}

pub fn gen_money(rng: &mut ThreadRng) -> String {
    let correct: bool = rng.gen::<f64>() > 0.01;
    if correct {
        (rng.gen::<f64>() * rng.gen::<i64>() as f64).to_string()
    } else {
        rand_string(rng)
    }
}

pub fn gen_type(rng: &mut ThreadRng) -> String {
    let correct: bool = rng.gen::<f64>() > 0.01;
    if correct {
        match rng.gen_range(0..=41) {
            0..=20 => "deposit".to_string(),
            21..=40 => "withdrawal".to_string(),
            41..=60 => "dispute".to_string(),
            61..=80 => "resolve".to_string(),
            81 => "chargeback".to_string(),
            _ => rand_string(rng)
        }
    } else {
        rand_string(rng)
    }
}

pub fn gen_line(rng: &mut ThreadRng) -> String {
    format!(
        "{}{}{},{}{}{},{}{}{},{}{}{}",
        gen_spaces(rng), gen_type(rng), gen_spaces(rng),
        gen_spaces(rng), gen_client(rng), gen_spaces(rng),
        gen_spaces(rng), gen_tx(rng), gen_spaces(rng),
        gen_spaces(rng), gen_money(rng), gen_spaces(rng),
    )
}

pub fn gen_json(rng: &mut ThreadRng) -> String {
    format!("{{\"type\": \"{}\",\"client\": {},\"tx\": {},\"amount\": {}}}", gen_type(rng), gen_client(rng), gen_tx(rng), gen_money(rng))
}