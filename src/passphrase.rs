use rand::seq::SliceRandom;

pub fn get_passphrase_from_str_wordlist(wordlist: &[&'static str]) -> String {
    let mut rng = rand::thread_rng();

    let mut passphrase = [""; 3];
    passphrase[0] = wordlist.choose(&mut rng).unwrap();
    passphrase[1] = wordlist.choose(&mut rng).unwrap();
    passphrase[2] = wordlist.choose(&mut rng).unwrap();

    passphrase.join("-")
}

pub fn get_passphrase_from_string_wordlist(wordlist: &[String]) -> String {
    let mut rng = rand::thread_rng();

    let mut passphrase: [String; 3] = Default::default();
    passphrase[0] = wordlist.choose(&mut rng).unwrap().to_string();
    passphrase[1] = wordlist.choose(&mut rng).unwrap().to_string();
    passphrase[2] = wordlist.choose(&mut rng).unwrap().to_string();

    passphrase.join("-")
}
