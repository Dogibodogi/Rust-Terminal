use std::fs;
use std::io::{self, Write};
use sysinfo::{System, SystemExt, ProcessExt, Pid};
use winreg::{HKEY, RegKey};
use winreg::enums::{HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER,HKEY_USERS, HKEY_CURRENT_CONFIG};
fn main() {
    loop {
        let comanda = citeste_comanda();
        if comanda == "exit" {
            break;
        }
        let argumente_comanda: Vec<&str> = comanda.split_whitespace().collect();
        if argumente_comanda.is_empty() {
            continue;
        }
        match argumente_comanda[0] {
            "cp" => copiaza_fisier(argumente_comanda),
            "mv" => muta_fisier(argumente_comanda),
            "rm" => sterge_fisier(argumente_comanda),
            "ps" => listeaza_procese(),
            "kill" => kill_proces(argumente_comanda),
            "lsreg"=>listeaza_toti_registrii(),
            "mkreg" =>  creeaza_registry_key(argumente_comanda),
            "rmreg" => rmreg(argumente_comanda),
            "modreg" => modreg(argumente_comanda),
            _ => println!("Comanda nu exista: {}", argumente_comanda[0]),
        }
    }
}
fn citeste_comanda() -> String {
    print!("rterm> ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    return input.trim().to_string();
}
fn copiaza_fisier(args: Vec<&str>) {
    if args.len() != 3 {
        println!("Numar incorect de argumente!\nUsage: cp <source> <destination>");
        return;
    }
    let cale_initiala = args[1];
    let cale_finala = args[2];

    match fs::metadata(cale_initiala) {
        Ok(metadata) => {
            if metadata.is_file() {
                if let Err(e) = fs::copy(cale_initiala, cale_finala) {
                    println!("Eroare la copierea fisierului: {}", e);
                } else {
                    println!("Fisier copiat cu succes.");
                }
            } else if metadata.is_dir() {
                if let Err(e) = copiaza_director(cale_initiala, cale_finala) {
                    println!("Eroare la copierea directorului: {}", e);
                } else {
                    println!("Director copiat cu succes.");
                }
            } else {
                println!("Tip necunoscut pentru sursa specificata.");
            }
        }
        Err(e) => println!("Eroare la accesarea sursei: {}", e),
    }
}
fn copiaza_director(cale_initiala: &str, cale_finala: &str) -> io::Result<()> {
    fs::create_dir_all(cale_finala)?;

    for entry in fs::read_dir(cale_initiala)? {
        let entry = entry?;
        let cale = entry.path();
        let dest_path = format!("{}/{}", cale_finala, entry.file_name().to_string_lossy());

        if cale.is_dir() {
            copiaza_director(&cale.to_string_lossy(), &dest_path)?;
        } else {
            fs::copy(&cale, &dest_path)?;
        }
    }
    Ok(())
}
fn muta_fisier(args: Vec<&str>) {
    if args.len() != 3 {
        println!("Numar incorect de argumente!\nUsage: cp <source> <destination>");
        return;
    }
    let cale_initiala = args[1];
    let cale_finala = args[2];
    if let Err(e) = fs::rename(cale_initiala, cale_finala) {
        println!("Eroare: {}", e);
    }
}
fn sterge_fisier(args: Vec<&str>) {
    if args.len() != 2 {
        println!("Numar incorect de argumente!\nUsage: rm <path>");
        return;
    }
    let path = args[1];
    match fs::metadata(path) {
        Ok(metadata) => {
            if metadata.is_file() {
                if let Err(e) = fs::remove_file(path) {
                    println!("Eroare: {}", e);
                } else {
                 println!("Fisier sters cu succes.");
                }
            } else if metadata.is_dir() {
                if let Err(e) = sterge_director(path) {
                    println!("Eroare la stergerea directorului: {}", e);
                } else {
                    println!("Director sters cu succes.");
                }
            } else {
                println!("Tip necunoscut pentru sursa specificata.");
            }
        }
        Err(e) => println!("Eroare la accesarea sursei: {}", e),
    }
}
fn sterge_director(path:&str)-> io::Result<()>
{
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let cale = entry.path();

        if cale.is_dir() {
            sterge_director(&cale.to_string_lossy())?;
        } else {
            fs::remove_file(&cale)?;
        }
    }
    fs::remove_dir(path)?;
    Ok(())
}
fn listeaza_procese() {

    let mut system = System::new_all();
    system.refresh_all();

    println!("{:<10} {:<30} {:<15}", "PID", "Nume Proces", "Memorie (KB)");
    println!("{}", "-".repeat(55));
    for (pid, process) in system.processes() {
        println!(
            "{:<10} {:<30} {:<15}",
            pid,
            process.name(),
            process.memory()
        );
    }
}
fn kill_proces(args: Vec<&str>) {
    if args.len() != 2 {
        println!("Numar incorect de argumente!\nUsage: kill <PID>");
        return;
    }

    let pid: i32 = match args[1].parse() {
        Ok(num) => num,
        Err(_) => {
            println!("PID invalid: {}", args[1]);
            return;
        }
    };

    let mut system = System::new_all();
    system.refresh_all();

    if let Some(process) = system.process(Pid::from(pid as usize)) {
        if process.kill() {
            println!("Procesul cu PID {} a fost terminat.", pid);
        } else {
            println!("Eroare: Procesul cu PID {} nu a putut fi terminat.", pid);
        }
    } else {
        println!("Eroare: Procesul cu PID {} nu a fost găsit.", pid);
    }
}
fn listeaza_toti_registrii() {
    println!("Afișez toate cheile de registru din HKEY_LOCAL_MACHINE și HKEY_CURRENT_USER");

    println!("Cheile din HKEY_LOCAL_MACHINE:");
    if let Err(e) = listeaza_subcheile(HKEY_LOCAL_MACHINE) {
        println!("Eroare la listarea cheilor din HKEY_LOCAL_MACHINE: {}", e);
    }
    println!("\nCheile din HKEY_CURRENT_USER:");
    if let Err(e) = listeaza_subcheile(HKEY_CURRENT_USER) {
        println!("Eroare la listarea cheilor din HKEY_CURRENT_USER: {}", e);
    }
    println!("\nCheile din HKEY_CURRENT_CONFIG:");
    if let Err(e) = listeaza_subcheile(HKEY_CURRENT_CONFIG) {
        println!("Eroare la listarea cheilor din HKEY_CURRENT_CONFIG: {}", e);
    }
    println!("\nCheile din HKEY_USERS:");
    if let Err(e) = listeaza_subcheile(HKEY_USERS) {
        println!("Eroare la listarea cheilor din HKEY_USERS: {}", e);
    }
}
fn listeaza_subcheile(root_key: HKEY) -> Result<(), Box<dyn std::error::Error>> {
    let root = RegKey::predef(root_key);

    for subkey in root.enum_keys() {
        match subkey {
            Ok(name) => {
                println!("- {}", name);
            }
            Err(e) => {
                println!("Eroare la accesarea unei chei: {}", e);
            }
        }
    }
    Ok(())
}
fn creeaza_registry_key(args: Vec<&str>){
    if args.len() != 2 {
        println!("Numar incorect de argumente!\nUsage: mkreg path");
        return;
    }
    let cale=args[1];
    let parts: Vec<&str> = cale.splitn(2, '\\').collect();
    if parts.len() < 2 {
        println!("Cale invalida. Utilizare: mkreg <root_key\\subkey>");
        return;
    }

    let root_key = match parts[0].to_uppercase().as_str() {
        "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
        "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
        _ => {
            println!("Cheie root invalida. Doar HKEY_LOCAL_MACHINE și HKEY_CURRENT_USER sunt suportate.");
            return;
        }
    };

    match root_key.create_subkey(parts[1]) {
        Ok(_) => println!("Cheia de registru '{}' a fost creată cu succes!", cale),
        Err(e) => println!("Eroare la crearea cheii de registru '{}': {}", cale, e),
    }
}
fn rmreg(args: Vec<&str>) {
    if args.len() != 2 {
        println!("Numar incorect de argumente!\nUsage: rmreg <path>");
        return;
    }

    let key_path = args[1];
    let parts: Vec<&str> = key_path.splitn(2, '\\').collect();
    
    if parts.len() < 2 {
        println!("Cale invalida. Utilizare: rmreg <root_key\\subkey>");
        return;
    }

    let root_key = match parts[0].to_uppercase().as_str() {
        "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
        "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
        _ => {
            println!("Cheie root invalida. Doar HKEY_LOCAL_MACHINE și HKEY_CURRENT_USER sunt suportate.");
            return;
        }
    };

    let subkey = parts[1];

    match root_key.open_subkey(subkey) {
        Ok(_) => {
            match root_key.delete_subkey(subkey) {
                Ok(_) => println!("Cheia de registru '{}' a fost stearsa cu succes!", key_path),
                Err(e) => println!("Eroare la stergerea cheii '{}': {}", key_path, e),
            }
        },
        Err(_) => {
            println!("Cheia de registru '{}' nu exista sau nu poate fi accesata.", key_path);
        }
    }
}
fn modreg(args: Vec<&str>) {
    if args.len() < 4 {
        println!("Numar incorect de argumente!\nUsage: modreg <root_key\\subkey> <value_name> <value_type> <value>");
        return;
    }
    let key_path = args[1];
    let value_name = args[2];
    let value_type = args[3].to_lowercase();
    let value_data = args[4];

    let parts: Vec<&str> = key_path.splitn(2, '\\').collect();
    if parts.len() < 2 {
        println!("Cale invalida. Utilizare: modreg <root_key\\subkey>");
        return;
    }

    let root_key = match parts[0].to_uppercase().as_str() {
        "HKEY_LOCAL_MACHINE" => RegKey::predef(HKEY_LOCAL_MACHINE),
        "HKEY_CURRENT_USER" => RegKey::predef(HKEY_CURRENT_USER),
        _ => {
            println!("Cheie root invalida. Doar HKEY_LOCAL_MACHINE și HKEY_CURRENT_USER sunt suportate.");
            return;
        }
    };

    let subkey = parts[1];

    let key = match root_key.create_subkey(subkey) {
        Ok((key, _)) => key,
        Err(e) => {
            println!("Eroare la deschiderea sau crearea cheii '{}': {}", key_path, e);
            return;
        }
    };

    match value_type.as_str() {
        "string" => {
            if let Err(e) = key.set_value(value_name, &value_data) {
                println!("Eroare la setarea valorii '{}': {}", value_name, e);
            } else {
                println!("Valoarea '{}' a fost setata cu succes in '{}'.", value_name, key_path);
            }
        }
        "dword" => {
            match value_data.parse::<u32>() {
                Ok(parsed_value) => {
                    if let Err(e) = key.set_value(value_name, &parsed_value) {
                        println!("Eroare la setarea valorii '{}': {}", value_name, e);
                    } else {
                        println!("Valoarea '{}' a fost setata cu succes in '{}'.", value_name, key_path);
                    }
                }
                Err(_) => println!("Valoare invalida pentru tipul DWORD."),
            }
        }
        "qword" => {
            match value_data.parse::<u64>() {
                Ok(parsed_value) => {
                    if let Err(e) = key.set_value(value_name, &parsed_value) {
                        println!("Eroare la setarea valorii '{}': {}", value_name, e);
                    } else {
                        println!("Valoarea '{}' a fost setata cu succes in '{}'.", value_name, key_path);
                    }
                }
                Err(_) => println!("Valoare invalida pentru tipul QWORD."),
            }
        }
        _ => {
            println!("Tip de valoare necunoscut: {}. Suportate: string, dword, qword.", value_type);
        }
    }
}
