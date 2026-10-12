/* ea:   0x000635E8 */
/* name: sub_635E8 */
/* size: 160 */
int __fastcall sub_635E8(_WORD *a1, int a2, int *a3)
{
  int v4; // r7
  int result; // r0
  unsigned int v6; // r6
  int v7; // r9
  _WORD *v8; // r5
  int v9; // r8
  _WORD *v10; // r4
  int v11; // r2
  unsigned int v12; // r3
  unsigned __int16 *v13; // r5
  __int16 v14; // t1
  _WORD *v15; // r4
  __int16 v16; // t1
  _WORD *v17; // r4
  int v18; // t1
  _WORD *v19; // r4

  v4 = a3[2] >> 1;
  result = *a3;
  v6 = (unsigned int)*a3 >> 2;
  if ( a2 == 0 )
  {
    __debugbreak();
    __debugbreak();
  }
  v7 = a3[1];
  v8 = *(_WORD **)(a2 + 28);
  if ( v7 != 0 )
  {
    v9 = 2 * (a3[3] >> 1);
    do
    {
      v10 = a1;
      if ( v6 != 0 )
      {
        v11 = v4;
        v12 = v6;
        do
        {
          v14 = *v8;
          v13 = v8 + 1;
          --v12;
          *v10 = v14;
          v15 = &v10[v11];
          v16 = *v13++;
          *v15 = v16;
          v17 = &v15[v11];
          v18 = *v13++;
          result = v18;
          *v17 = v18;
          v19 = &v17[v11];
          LOWORD(v18) = *v13;
          v8 = v13 + 1;
          *v19 = v18;
          v10 = &v19[v11];
        }
        while ( v12 != 0 );
      }
      --v7;
      a1 = (_WORD *)((char *)a1 + v9);
    }
    while ( v7 != 0 );
  }
  return result;
}

