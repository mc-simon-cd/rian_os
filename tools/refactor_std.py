import os
import re

def refactor_file(path):
    if not path.endswith('.rs'): return
    with open(path, 'r') as f:
        content = f.read()

    original = content
    content = content.replace('std::sync::Mutex;', 'spin::Mutex;')
    content = content.replace('std::sync::Arc;', 'alloc::sync::Arc;')
    content = content.replace('std::collections::HashMap', 'alloc::collections::BTreeMap')
    content = content.replace('HashMap<', 'BTreeMap<')
    content = content.replace('HashMap::new()', 'BTreeMap::new()')
    content = content.replace('std::sync::atomic', 'core::sync::atomic')
    content = content.replace('std::convert::TryInto', 'core::convert::TryInto')
    content = content.replace('std::cmp::min', 'core::cmp::min')

    content = content.replace('std::string::String', 'alloc::string::String')
    content = content.replace('std::vec::Vec', 'alloc::vec::Vec')
    content = content.replace('std::time', 'core::time') 
    
    content = content.replace('use std::io::Write;', '')
    content = content.replace('use std::io::Read;', '')
    content = content.replace('std::io::stdout().flush().unwrap();', '')

    content = re.sub(r'(?<!crate::)println!', 'crate::println!', content)
    content = re.sub(r'(?<!crate::)print!', 'crate::print!', content)
    
    if original != content:
        if 'alloc::' in content and 'extern crate alloc;' not in content and 'main.rs' not in path:
            content = "extern crate alloc;\n" + content
        with open(path, 'w') as f:
            f.write(content)
        print(f"Refactored: {path}")

for root, _, files in os.walk('/home/can/Masaüstü/calişma/projeler/rian_cekirdek/src'):
    for file in files:
        refactor_file(os.path.join(root, file))
