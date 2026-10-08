/* ea:   0x00045234 */
/* name: sub_45234 */
/* size: 392 */
int __fastcall sub_45234(int result, int *a2, unsigned int a3, int *a4)
{
  int v4; // r8
  int *v5; // r7
  int v7; // r2
  int v8; // r0
  int v9; // r5
  unsigned int v10; // r3
  int *v11; // r6
  int v12; // r4
  int v13; // r3
  int *v14; // r2
  int *v15; // r1
  int v16; // r2
  int *v17; // r0
  unsigned int v18; // r1
  int v19; // r0
  int *v20; // r1
  int *v21; // r2
  char v22; // [sp+8h] [bp-1Ch] BYREF

  v4 = result;
  v5 = a2;
  if ( a3 != 0 )
  {
    v7 = *(_DWORD *)(result + 4);
    if ( a3 > (*(_DWORD *)(result + 8) - v7) >> 2 )
    {
      return sub_12F60((int **)result, a2, a4, (int)&v22, a3, 0);
    }
    else
    {
      v8 = v7 - (_DWORD)a2;
      v9 = *a4;
      v10 = (v7 - (int)a2) >> 2;
      v11 = *(int **)(v4 + 4);
      if ( v10 <= a3 )
      {
        v17 = *(int **)(v4 + 4);
        if ( a3 != v10 )
        {
          v18 = a3 - v10;
          do
          {
            --v18;
            if ( v17 != nullptr )
              *v17 = v9;
            ++v17;
          }
          while ( v18 != 0 );
        }
        v19 = *(_DWORD *)(v4 + 4);
        v20 = v5;
        v21 = (int *)(v19 + 4 * (a3 - v10));
        for ( *(_DWORD *)(v4 + 4) = v21; v20 != v11; ++v21 )
        {
          if ( v21 != nullptr )
            v19 = *v20;
          ++v20;
          if ( v21 != nullptr )
            *v21 = v19;
        }
        result = *(_DWORD *)(v4 + 4);
        for ( *(_DWORD *)(v4 + 4) = result + 4 * v10; v5 != v11; ++v5 )
          *v5 = v9;
      }
      else
      {
        v12 = 4 * a3;
        v13 = v7 - v12;
        v14 = (int *)(v7 - v12);
        v15 = *(int **)(v4 + 4);
        if ( &v11[v12 / 0xFFFFFFFC] != v11 )
        {
          do
          {
            if ( v15 != nullptr )
              v8 = *v14;
            ++v14;
            if ( v15 != nullptr )
              *v15 = v8;
            ++v15;
          }
          while ( v14 != v11 );
        }
        *(_DWORD *)(v4 + 4) += v12;
        v16 = (v13 - (int)v5) >> 2;
        if ( v16 > 0 )
        {
          do
          {
            --v11;
            --v16;
            *v11 = v11[v12 / 0xFFFFFFFC];
          }
          while ( v16 != 0 );
        }
        for ( result = (int)&v5[v12 / 4u]; v5 != (int *)result; ++v5 )
          *v5 = v9;
      }
    }
  }
  return result;
}

