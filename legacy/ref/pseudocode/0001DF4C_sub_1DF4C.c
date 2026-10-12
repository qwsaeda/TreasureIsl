/* ea:   0x0001DF4C */
/* name: sub_1DF4C */
/* size: 328 */
_DWORD *__fastcall sub_1DF4C(_DWORD *result, _DWORD *a2, unsigned int a3, int *a4)
{
  _DWORD *v4; // r9
  _DWORD *v7; // r2
  int v8; // r1
  int v9; // r6
  unsigned int v10; // r4
  _BYTE *v11; // r7
  size_t v12; // r5
  char *v13; // r4
  unsigned int v14; // r3
  void *v15; // r0
  char v16; // [sp+8h] [bp-20h] BYREF

  v4 = result;
  if ( a3 != 0 )
  {
    v7 = (_DWORD *)result[1];
    v8 = result[2] - (_DWORD)v7;
    if ( a3 > v8 >> 2 )
    {
      return (_DWORD *)sub_1E4A8(result, a2, a4, &v16, a3, 0);
    }
    else
    {
      v9 = *a4;
      v10 = v7 - a2;
      v11 = (_BYTE *)result[1];
      if ( v10 <= a3 )
      {
        v14 = a3 - v10;
        if ( a3 != v10 && v14 != 0 )
        {
          v8 = (int)&v11[4 * v14];
          do
            *v7++ = v9;
          while ( v7 != (_DWORD *)v8 );
        }
        if ( v11 != (_BYTE *)a2 )
          v7 = (_DWORD *)(v11 - (_BYTE *)a2);
        v15 = (void *)(result[1] + 4 * v14);
        if ( v11 != (_BYTE *)a2 )
          v8 = (int)a2;
        v4[1] = v15;
        if ( v11 != (_BYTE *)a2 )
          memmove(v15, (const void *)v8, (size_t)v7);
        result = (_DWORD *)v4[1];
        for ( v4[1] = &result[v10]; a2 != (_DWORD *)v11; ++a2 )
          *a2 = v9;
      }
      else
      {
        v12 = 4 * a3;
        v13 = (char *)&v7[v12 / 0xFFFFFFFC];
        if ( v7 != &v7[v12 / 0xFFFFFFFC] )
          memmove(v11, &v11[-v12], v12);
        v4[1] += v12;
        if ( v13 - (char *)a2 > 0 )
          memmove(&a2[v12 / 4], a2, v13 - (char *)a2);
        for ( result = &a2[v12 / 4]; a2 != result; ++a2 )
          *a2 = v9;
      }
    }
  }
  return result;
}

