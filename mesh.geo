// mesh.geo
SetFactory("OpenCASCADE");
R0 = 1.4;
a = 0.9;
Rmin = R0 - 1.8*a;
Rmax = R0 + 1.8*a;
Zmin = -1.8*a;
Zmax = 1.8*a;
Rectangle(1) = {Rmin, Zmin, 0, Rmax-Rmin, Zmax-Zmin};
Physical Surface(1) = {1}; // domínio
Physical Curve(2) = {1,2,3,4}; // fronteira
