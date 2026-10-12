/* ea:   0x0001831C */
/* name: sub_1831C */
/* size: 52 */
int __fastcall sub_1831C(int a1, int a2)
{
  int v3; // r3

  v3 = sub_1836C(a1, 1);
  if ( v3 != -16 )
  {
    *(_WORD *)(v3 + 16) = *(_WORD *)a2;
    *(_DWORD *)(v3 + 20) = *(_DWORD *)(a2 + 4);
  }
  return v3;
}

