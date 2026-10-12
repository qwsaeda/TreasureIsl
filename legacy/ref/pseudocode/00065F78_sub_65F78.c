/* ea:   0x00065F78 */
/* name: sub_65F78 */
/* size: 108 */
int __fastcall sub_65F78(int *a1, int a2)
{
  int v4; // r0
  __int16 **v6; // r0
  void *v7[6]; // [sp+8h] [bp-18h] BYREF

  v4 = sub_1337C(a1, 92, -1);
  if ( v4 == -1 )
    return 0;
  v6 = (__int16 **)sub_60098(a1, (int)v7, 0, v4 + 1);
  if ( v6 != (__int16 **)a2 )
    sub_17890(a2, *v6, v6[1]);
  sub_12F48(v7);
  return 1;
}

