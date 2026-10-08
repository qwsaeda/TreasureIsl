/* ea:   0x00063A58 */
/* name: sub_63A58 */
/* size: 92 */
int __fastcall sub_63A58(int a1, _DWORD *a2)
{
  int v4; // r0
  int v5; // r1
  int v6; // r2
  int v7; // r1

  v4 = *(_DWORD *)(a1 + 8);
  if ( v4 == 0 )
    __debugbreak();
  *a2 = *(_DWORD *)(v4 + 4);
  v5 = *(_DWORD *)(a1 + 8);
  if ( v5 == 0 )
    __debugbreak();
  v6 = *(_DWORD *)(v5 + 8);
  v7 = 2 * *a2;
  a2[2] = 2;
  a2[1] = v6;
  a2[3] = v7;
  a2[4] = 0;
  return 0;
}

