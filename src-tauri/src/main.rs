// Impede a abertura de um console extra no Windows em builds de release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    nr_manager_lib::run()
}
