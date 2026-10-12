/* ea:   0x00036284 */
/* name: sub_36284 */
/* size: 196 */
int __fastcall sub_36284(int a1, int a2, int a3)
{
  int v6; // r8
  int v7; // r6
  char *i; // r5
  int v9; // r4
  char v10; // r0

  v6 = sub_1831C(a1, a2 + 16);
  *(_BYTE *)v6 = *(_BYTE *)a2;
  *(_DWORD *)(v6 + 12) = 0;
  *(_DWORD *)(v6 + 4) = a3;
  *(_DWORD *)(v6 + 8) = 0;
  if ( *(_DWORD *)(a2 + 12) != 0 )
    *(_DWORD *)(v6 + 12) = sub_36284(a1, *(_DWORD *)(a2 + 12), v6);
  v7 = v6;
  for ( i = *(char **)(a2 + 8); i != nullptr; i = *((char **)i + 2) )
  {
    v9 = sub_1831C(a1, (int)(i + 16));
    v10 = *i;
    *(_DWORD *)(v9 + 8) = 0;
    *(_BYTE *)v9 = v10;
    *(_DWORD *)(v9 + 12) = 0;
    *(_DWORD *)(v7 + 8) = v9;
    *(_DWORD *)(v9 + 4) = v7;
    if ( *((_DWORD *)i + 3) != 0 )
      *(_DWORD *)(v9 + 12) = sub_36284(a1, *((_DWORD *)i + 3), v9);
    v7 = v9;
  }
  return v6;
}

