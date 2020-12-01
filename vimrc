map z :w!%<cr>:!gcc % && ./a.out<cr>
map z :w!%<cr>:!cargo run && echo -en "[=>$?]"<cr>
map z :w!%<cr>:!rustc -o a % && ( ./a ; echo -e "\n$?" )<cr>
map z :w!%<cr>:!./%<cr>
map z :w!%<cr>:!rr<cr>
