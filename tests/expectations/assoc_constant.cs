using System.Runtime.InteropServices;
static class Imports {
const string DLL = ".dll";
struct Foo {

};
const int Foo_GA = 10;
const float Foo_ZO = 3.14f;

[DllImport(DLL)]
static extern void root(Foo x);
}
