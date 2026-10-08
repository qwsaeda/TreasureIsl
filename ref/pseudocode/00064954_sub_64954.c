/* ea:   0x00064954 */
/* name: sub_64954 */
/* size: 556 */
void *__fastcall sub_64954(int a1, int a2, int a3, _DWORD *a4)
{
  char v7; // r4
  _DWORD *v8; // r6
  _BYTE *v10; // r5
  _WORD *v11; // r3
  int i; // r4
  _BYTE *v13; // r2
  int v14; // r3
  char v15; // r0
  int v16; // r5
  unsigned int v17; // r4
  _BYTE *v18; // r8
  _BYTE *v19; // r6
  unsigned int v20; // r7
  _BYTE *v21; // r3
  unsigned int j; // r2
  unsigned int v23; // r2
  unsigned int v24; // t1
  void *v25; // [sp+8h] [bp-2Ch]

  if ( a2 == 0 || *a4 != 4 )
    __debugbreak();
  v7 = 0;
  if ( *(_BYTE *)(a1 + 12) != 0 )
    v7 = 2;
  v8 = sub_62694(a4[1], a4[2], 8, v7);
  v25 = v8;
  if ( v8 == nullptr )
    return nullptr;
  v10 = (_BYTE *)(a2 + 16);
  if ( v7 == 2 )
  {
    v11 = (_WORD *)v8[8];
    for ( i = 16; i != 0; --i )
    {
      *v11++ = (8 * (v10[1] & 0xFC | (32 * (*v10 & 0xF8)))) | ((unsigned __int8)v10[2] >> 3);
      v10 += 3;
    }
  }
  else
  {
    v13 = (_BYTE *)v8[8];
    v14 = 16;
    do
    {
      --v14;
      *v13 = *v10;
      v13[1] = v10[1];
      v15 = v10[2];
      v10 += 3;
      v13[2] = v15;
      v13 += 3;
    }
    while ( v14 != 0 );
  }
  v16 = a2 + 128;
  v17 = *(unsigned __int16 *)(a2 + 66) * *(unsigned __int8 *)(a2 + 65);
  v18 = operator new(v17);
  if ( v18 != nullptr )
  {
    v19 = (_BYTE *)v8[7];
    v20 = 0;
    if ( a4[2] != 0 )
    {
      while ( 1 )
      {
        v16 = sub_64CFC(a1, v18, v17, v16, a2, a3);
        if ( v16 == 0 )
          break;
        v21 = v18;
        *v19 = *v18 >> 4;
        for ( j = 1; j < a4[1]; j = v23 + 1 )
        {
          v19[j] = *v21 & 0xF;
          v23 = j + 1;
          if ( v23 >= a4[1] )
            break;
          v24 = (unsigned __int8)*++v21;
          v19[v23] = v24 >> 4;
        }
        ++v20;
        v19 += a4[1];
        if ( v20 >= a4[2] )
          goto LABEL_22;
      }
      free(v25);
      operator delete(v18);
      return nullptr;
    }
    else
    {
LABEL_22:
      operator delete(v18);
      return v25;
    }
  }
  else
  {
    free(v8);
    return nullptr;
  }
}

