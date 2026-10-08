/* ea:   0x00036384 */
/* name: sub_36384 */
/* size: 348 */
int __fastcall sub_36384(int result, int **a2)
{
  int v2; // r9
  int *v3; // r6
  int *v4; // r5
  int *v5; // r3
  unsigned int v6; // r8
  _BYTE *v7; // r0
  _BYTE *v8; // r4
  int v9; // r2
  bool v10; // zf
  _DWORD *v11; // r1
  unsigned int v12; // r2
  int v13; // r1
  int v14; // t1
  int v15; // r2
  int v16; // t1
  int *v17; // r4
  int v18; // r0
  int *v19; // r5
  int *i; // r3

  v2 = result;
  if ( a2 == (int **)result )
    return v2;
  v3 = *a2;
  v4 = a2[1];
  v5 = *(int **)result;
  v6 = v4 - *a2;
  if ( v6 <= (*(_DWORD *)(result + 8) - *(_DWORD *)result) >> 2 )
  {
    v12 = (*(_DWORD *)(result + 4) - (int)v5) >> 2;
    if ( v12 < v6 )
    {
      v15 = (int)(4 * v12) >> 2;
      if ( v15 > 0 )
      {
        do
        {
          v16 = *v3++;
          --v15;
          *v5++ = v16;
        }
        while ( v15 != 0 );
      }
      v17 = *(int **)(result + 4);
      v18 = (int)v17 - *(_DWORD *)result;
      v19 = a2[1];
      for ( i = &(*a2)[v18 >> 2]; i != v19; ++v17 )
      {
        if ( v17 != nullptr )
          v18 = *i;
        ++i;
        if ( v17 != nullptr )
          *v17 = v18;
      }
    }
    else
    {
      v13 = v4 - v3;
      if ( v13 > 0 )
      {
        do
        {
          v14 = *v3++;
          --v13;
          *v5++ = v14;
        }
        while ( v13 != 0 );
        *(_DWORD *)(result + 4) = *(_DWORD *)result + 4 * v6;
        return result;
      }
    }
    *(_DWORD *)(v2 + 4) = *(_DWORD *)v2 + 4 * v6;
    return v2;
  }
  v7 = sub_32DCC(result + 8, v4 - *a2);
  v8 = v7;
  if ( v3 != v4 )
  {
    v9 = v7 - (_BYTE *)v3;
    do
    {
      v11 = (int *)((char *)v3 + v9);
      v10 = (int *)((char *)v3 + v9) == nullptr;
      if ( (int *)((char *)v3 + v9) != nullptr )
        v7 = (_BYTE *)*v3;
      ++v3;
      if ( !v10 )
        *v11 = v7;
    }
    while ( v3 != v4 );
  }
  sub_32A00((void **)v2);
  *(_DWORD *)v2 = v8;
  *(_DWORD *)(v2 + 8) = &v8[4 * v6];
  *(_DWORD *)(v2 + 4) = &v8[4 * v6];
  return v2;
}

