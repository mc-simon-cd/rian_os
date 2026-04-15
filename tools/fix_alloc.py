import os
import re

def fix_file(path):
    if not path.endswith('.rs'): return
    with open(path, 'r') as f:
        content = f.read()
    
    original = content
    
    content = content.replace('std::str::from_utf8', 'core::str::from_utf8')
    content = content.replace('std::array::', 'core::array::')
    content = content.replace('std::ops::', 'core::ops::')

    imports = set()
    
    if re.search(r'\bString\b', content) and 'use alloc::string::String;' not in content:
        imports.add('use alloc::string::String;')
    if re.search(r'\bVec\b', content) and 'use alloc::vec::Vec;' not in content:
        imports.add('use alloc::vec::Vec;')
    if re.search(r'\bformat!\b', content) and 'use alloc::format;' not in content:
        imports.add('use alloc::format;')
    if re.search(r'\bvec!\b', content) and 'use alloc::vec;' not in content:
        imports.add('use alloc::vec;')
    if re.search(r'\bBTreeMap\b', content) and 'use alloc::collections::BTreeMap;' not in content:
        imports.add('use alloc::collections::BTreeMap;')

    if imports:
        if 'extern crate alloc;' not in content and 'main.rs' not in path:
            imports.add('extern crate alloc;')
            
        import_str = '\n'.join(imports) + '\n'
        
        parts = content.split('// -----------------------------------------------------------------------------', 2)
        if len(parts) == 3:
            content = parts[0] + '// -----------------------------------------------------------------------------' + parts[1] + '// -----------------------------------------------------------------------------\n' + import_str + parts[2]
        else:
            content = import_str + content

    if original != content:
        with open(path, 'w') as f:
            f.write(content)
        print(f"Fixed missing bare-metal imports in: {path}")

for root, _, files in os.walk('/home/can/Masaüstü/calişma/projeler/rian_cekirdek/src'):
    for file in files:
        fix_file(os.path.join(root, file))
