/* ea:   0x00036EA4 */
/* name: sub_36EA4 */
/* size: 336 */
int __fastcall sub_36EA4(int result, char *a2, unsigned int a3, int *a4)
{
  int v4; // r9
  char *v5; // r8
  char *v7; // r2
  int v8; // r6
  unsigned int v9; // r4
  char *v10; // r7
  int v11; // r5
  char *v12; // r4
  unsigned int v13; // r3
  char v14; // [sp+9h] [bp-1Fh] BYREF

  v4 = result;
  v5 = a2;
  if ( a3 != 0 )
  {
    v7 = *(char **)(result + 4);
    if ( a3 > (*(_DWORD *)(result + 8) - (int)v7) >> 2 )
    {
      return sub_3854C(result, a2, a4, &v14, a3, 0);
    }
    else
    {
      v8 = *a4;
      v9 = (v7 - a2) >> 2;
      v10 = *(char **)(result + 4);
      if ( v9 <= a3 )
      {
        v13 = a3 - v9;
        if ( a3 != v9 && v13 != 0 )
        {
          do
          {
            *(_DWORD *)v7 = v8;
            v7 += 4;
          }
          while ( v7 != &v10[4 * v13] );
        }
        *(_DWORD *)(result + 4) += 4 * v13;
        sub_3900C(a2, v10);
        result = *(_DWORD *)(v4 + 4) + 4 * v9;
        for ( *(_DWORD *)(v4 + 4) = result; v5 != v10; v5 += 4 )
          *(_DWORD *)v5 = v8;
      }
      else
      {
        v11 = 4 * a3;
        v12 = &v7[-v11];
        sub_39E48(&v7[-v11], v7);
        *(_DWORD *)(v4 + 4) += v11;
        if ( v12 - v5 > 0 )
          memmove(&v10[-(v12 - v5)], v5, v12 - v5);
        for ( result = (int)&v5[v11]; v5 != (char *)result; v5 += 4 )
          *(_DWORD *)v5 = v8;
      }
    }
  }
  return result;
}

