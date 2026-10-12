/* ea:   0x0001109C */
/* name: sub_1109C */
/* size: 36 */
int __fastcall sub_1109C(_DWORD *a1, int a2, int a3, int a4)
{
  int result; // r0
  int v6; // r1

  a1[1] = a2;
  result = *(_DWORD *)(a3 + 4);
  a1[2] = result;
  v6 = *(_DWORD *)(a3 + 8);
  a1[4] = a4;
  a1[3] = v6;
  return result;
}

