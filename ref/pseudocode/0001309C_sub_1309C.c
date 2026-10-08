/* ea:   0x0001309C */
/* name: sub_1309C */
/* size: 116 */
int __fastcall sub_1309C(int a1)
{
  int v2; // r0
  WCHAR Filename[264]; // [sp+0h] [bp-210h] BYREF

  GetModuleFileNameW(nullptr, Filename, 0x104u);
  sub_12F04(Filename);
  sub_1318C(a1, Filename);
  v2 = sub_1337C(a1, 92, -1);
  if ( v2 == -1 )
    return 0;
  sub_13110(a1, v2 + 1, -1);
  return 1;
}

