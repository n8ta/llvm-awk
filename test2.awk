
BEGIN { x = 3; }
{ x = 0; }
x { print x; }
END { print x; }