/* ea:   0x0001E61C */
/* name: sub_1E61C */
/* size: 80 */
int __fastcall sub_1E61C(int a1)
{
  _DWORD *v2; // r1
  int v3; // r0

  *(_BYTE *)(a1 + 4) = 0;
  v2 = (_DWORD *)(a1 + 20);
  *(_DWORD *)(a1 + 8) = 0;
  v3 = a1 + 8;
  *(_DWORD *)(v3 + 4) = 0;
  *(_DWORD *)(v3 + 8) = 0;
  v2[5] = 0;
  *v2 = off_6D220;
  v2[1] = -1;
  v2[2] = 0;
  *(_DWORD *)a1 = off_6D1E4;
  return a1;
}

