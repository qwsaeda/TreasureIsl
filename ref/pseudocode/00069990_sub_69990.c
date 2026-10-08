/* ea:   0x00069990 */
/* name: sub_69990 */
/* size: 320 */
int __fastcall sub_69990(int a1, _DWORD *a2, int a3)
{
  int v5; // r10
  char *v6; // r6
  unsigned int v7; // r0
  unsigned int v8; // r1
  char *v9; // r5
  size_t v10; // r4
  int v11; // r0
  int v12; // r0
  char *v13; // r1
  char *v14; // r0
  char *v15; // r7
  char *v16; // r5
  size_t v17; // r1
  size_t v18; // r4
  int v19; // r0
  int v20; // r0

  v5 = a3;
  v6 = *(char **)(a1 + 48);
  v7 = *(_DWORD *)(a1 + 52);
  v8 = a2[4];
  v9 = (char *)a2[3];
  if ( (unsigned int)v6 > v7 )
    v7 = *(_DWORD *)(a1 + 44);
  v10 = v7 - (_DWORD)v6;
  if ( v7 - (unsigned int)v6 > v8 )
    v10 = v8;
  if ( v10 != 0 && a3 == -5 )
    v5 = 0;
  v11 = a2[5];
  a2[4] = v8 - v10;
  a2[5] = v11 + v10;
  if ( *(_DWORD *)(a1 + 56) != 0 )
  {
    v12 = (*(int (__fastcall **)(_DWORD, char *, size_t))(a1 + 56))(*(_DWORD *)(a1 + 60), v6, v10);
    *(_DWORD *)(a1 + 60) = v12;
    a2[12] = v12;
  }
  memcpy(v9, v6, v10);
  v13 = *(char **)(a1 + 44);
  v14 = &v6[v10];
  v15 = &v9[v10];
  if ( &v6[v10] == v13 )
  {
    v16 = *(char **)(a1 + 40);
    if ( *(char **)(a1 + 52) == v13 )
      *(_DWORD *)(a1 + 52) = v16;
    v17 = a2[4];
    v18 = *(_DWORD *)(a1 + 52) - (_DWORD)v16;
    if ( v18 > v17 )
      v18 = a2[4];
    if ( v18 != 0 && v5 == -5 )
      v5 = 0;
    v19 = a2[5];
    a2[4] = v17 - v18;
    a2[5] = v19 + v18;
    if ( *(_DWORD *)(a1 + 56) != 0 )
    {
      v20 = (*(int (__fastcall **)(_DWORD, char *, size_t))(a1 + 56))(*(_DWORD *)(a1 + 60), v16, v18);
      *(_DWORD *)(a1 + 60) = v20;
      a2[12] = v20;
    }
    memcpy(v15, v16, v18);
    v15 += v18;
    v14 = &v16[v18];
  }
  a2[3] = v15;
  *(_DWORD *)(a1 + 48) = v14;
  return v5;
}

