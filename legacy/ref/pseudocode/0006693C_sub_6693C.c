/* ea:   0x0006693C */
/* name: sub_6693C */
/* size: 92 */
BOOL __fastcall sub_6693C(int a1, _WORD *a2)
{
  bool v2; // r4
  void *v4[4]; // [sp+8h] [bp-24h] BYREF
  void *v5[5]; // [sp+18h] [bp-14h] BYREF

  if ( a2 == nullptr )
    __debugbreak();
  sub_16480(v5, a2);
  sub_1333C(v4);
  v2 = (unsigned __int8)sub_65F78((int *)v5, (int)v4) == 0;
  sub_12F48(v4);
  sub_12F48(v5);
  return v2;
}

