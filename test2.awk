END { if(x) { x = 4; } else { x = 100; } }
BEGIN { x = 1; } { x = 3; }
x { print x; }
END { print x; }