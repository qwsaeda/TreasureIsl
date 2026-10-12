/* ea:   0x00030EB8 */
/* name: sub_30EB8 */
/* size: 88 */
int __fastcall sub_30EB8(int a1, int a2)
{
  int *v3; // r0
  int v4; // r4
  int v6; // [sp+0h] [bp-1Ch] BYREF
  int v7; // [sp+4h] [bp-18h] BYREF
  char v8; // [sp+8h] [bp-14h] BYREF

  v3 = (int *)sub_33EE0(a1, &v8, a2);
  v6 = *v3;
  v7 = v3[1];
  v4 = sub_38030(&v6, &v7);
  sub_33D5C(a1, v6, v7);
  return v4;
}

