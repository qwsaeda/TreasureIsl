/* ea:   0x0001E5CC */
/* name: sub_1E5CC */
/* size: 80 */
int __fastcall sub_1E5CC(_WORD **a1)
{
  _WORD *v1; // r0
  int v2; // r4
  void *v4[5]; // [sp+8h] [bp-14h] BYREF

  v1 = *a1;
  if ( v1 == nullptr )
    __debugbreak();
  sub_16480(v4, v1);
  sub_18024(v4);
  v2 = sub_18094((unsigned __int16 *)v4[0]);
  sub_12F48(v4);
  return v2;
}

