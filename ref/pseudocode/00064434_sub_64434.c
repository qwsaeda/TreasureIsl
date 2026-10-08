/* ea:   0x00064434 */
/* name: sub_64434 */
/* size: 196 */
char *__fastcall sub_64434(char **a1, unsigned int a2)
{
  char *v4; // r6
  char *result; // r0
  char *v6; // r7
  int v7; // r9
  int v8; // r0
  char *v9; // r0
  char *v10; // r8
  int v11; // r4
  char *v12; // r5
  int v13; // r1
  int v14; // r2
  int v15; // r3
  char *v16; // r8

  v4 = *a1;
  result = a1[2];
  if ( a2 > (result - v4) >> 4 )
  {
    v6 = a1[1];
    v7 = (v6 - v4) >> 4;
    v8 = (int)(a1 + 2);
    if ( v4 != nullptr )
    {
      v9 = (char *)sub_64400(v8, a2);
      v10 = v9;
      if ( v4 != v6 )
      {
        v11 = v9 - v4;
        do
        {
          v12 = &v4[v11];
          if ( &v4[v11] != nullptr )
          {
            v13 = *((_DWORD *)v4 + 1);
            v14 = *((_DWORD *)v4 + 2);
            v15 = *((_DWORD *)v4 + 3);
            *(_DWORD *)v12 = *(_DWORD *)v4;
            *((_DWORD *)v12 + 1) = v13;
            *((_DWORD *)v12 + 2) = v14;
            *((_DWORD *)v12 + 3) = v15;
          }
          v4 += 16;
        }
        while ( v4 != v6 );
      }
      sub_643EC((void **)a1);
      *a1 = v10;
      result = &v10[16 * v7];
      a1[1] = result;
      a1[2] = &v10[16 * a2];
    }
    else
    {
      v16 = (char *)sub_64400(v8, a2);
      result = &v16[16 * v7];
      *a1 = v16;
      a1[1] = result;
      a1[2] = &v16[16 * a2];
    }
  }
  return result;
}

