/* ea:   0x0006374C */
/* name: sub_6374C */
/* size: 84 */
int __fastcall sub_6374C(_DWORD *a1, _DWORD *a2)
{
  int v4; // r0
  int v5; // r1
  int result; // r0

  v4 = a1[2];
  if ( v4 == 0 )
    __debugbreak();
  *a2 = *(_DWORD *)(v4 + 4);
  v5 = a1[2];
  if ( v5 == 0 )
    __debugbreak();
  a2[1] = *(_DWORD *)(v5 + 8);
  a2[2] = a1[9];
  result = a1[10];
  a2[4] = 0;
  a2[3] = result;
  return result;
}

