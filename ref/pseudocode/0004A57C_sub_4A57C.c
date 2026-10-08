/* ea:   0x0004A57C */
/* name: sub_4A57C */
/* size: 348 */
int *__fastcall sub_4A57C(int *result, int *a2)
{
  int *v2; // r9
  int *v3; // r6
  int *v4; // r5
  _DWORD *v5; // r3
  unsigned int v6; // r8
  int v7; // r0
  int v8; // r4
  int v9; // r2
  bool v10; // zf
  int *v11; // r1
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
  if ( a2 == result )
    return v2;
  v3 = (int *)*a2;
  v4 = (int *)a2[1];
  v5 = (_DWORD *)*result;
  v6 = ((int)v4 - *a2) >> 2;
  if ( v6 <= (result[2] - *result) >> 2 )
  {
    v12 = (result[1] - (int)v5) >> 2;
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
      v17 = (int *)result[1];
      v18 = (int)v17 - *result;
      v19 = (int *)a2[1];
      for ( i = (int *)(*a2 + 4 * (v18 >> 2)); i != v19; ++v17 )
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
        result[1] = *result + 4 * v6;
        return result;
      }
    }
    v2[1] = *v2 + 4 * v6;
    return v2;
  }
  v7 = sub_4A7A8(result + 2, ((int)v4 - *a2) >> 2);
  v8 = v7;
  if ( v3 != v4 )
  {
    v9 = v7 - (_DWORD)v3;
    do
    {
      v11 = (int *)((char *)v3 + v9);
      v10 = (int *)((char *)v3 + v9) == nullptr;
      if ( (int *)((char *)v3 + v9) != nullptr )
        v7 = *v3;
      ++v3;
      if ( !v10 )
        *v11 = v7;
    }
    while ( v3 != v4 );
  }
  sub_4A794(v2);
  *v2 = v8;
  v2[2] = v8 + 4 * v6;
  v2[1] = v8 + 4 * v6;
  return v2;
}

