// -*- coding: utf-8 -*-

fn main() {
    embuild::build::CfgArgs::output_propagated("ESP_IDF").unwrap();
    embuild::build::LinkArgs::output_propagated("ESP_IDF").unwrap();
}

// vim: ts=4 sw=4 expandtab
