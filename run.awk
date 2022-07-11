function effect() {
    print "side effect1 ran" a;
    a = a + 1;
    return 1;
}
function effect2() {
    print "side effect2 ran" a;
    a = a + 1;
    return 0;
}
function effect3() {
    print "side effect3 ran" a;
    a = a + 1;
    return 0;
}


BEGIN {
print effect() && effect2() && effect3();
print 1;
}