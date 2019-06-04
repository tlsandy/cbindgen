using System.Runtime.InteropServices;
static class Imports {
const string DLL = ".dll";
struct A {
  int x;
  float y;
};

struct B {
  A data;
};
}
