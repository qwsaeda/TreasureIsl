/* ea:   0x00019A4C */
/* name: sub_19A4C */
/* size: 76 */
int __fastcall sub_19A4C(_WORD *a1)
{
  int v1; // r4
  void *v3[5]; // [sp+8h] [bp-14h] BYREF

  if ( a1 == nullptr )
    __debugbreak();
  sub_16480(v3, a1);
  sub_18024(v3);
  v1 = sub_18094((unsigned __int16 *)v3[0]);
  sub_12F48(v3);
  return v1;
}

