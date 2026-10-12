/* ea:   0x0006475C */
/* name: sub_6475C */
/* size: 504 */
_DWORD *__fastcall sub_6475C(int a1, int a2, int a3, int *a4)
{
  _BYTE *v6; // r5
  char v7; // r4
  _DWORD *v8; // r9
  _WORD *v9; // r3
  int i; // r4
  _BYTE *v11; // r2
  int v12; // r3
  char v13; // r0
  int v14; // r7
  unsigned int v15; // r6
  void *v16; // r8
  char *v18; // r4
  unsigned int v19; // r5

  if ( a2 == 0 || *a4 != 8 )
    __debugbreak();
  v6 = (_BYTE *)(a2 + a3 - 768);
  if ( *(_BYTE *)(a2 + a3 - 769) == 12 )
  {
    v7 = 0;
    if ( *(_BYTE *)(a1 + 12) != 0 )
      v7 = 2;
    v8 = sub_62694(a4[1], a4[2], *a4, v7);
    if ( v8 != nullptr )
    {
      if ( v7 == 2 )
      {
        v9 = (_WORD *)v8[8];
        for ( i = 256; i != 0; --i )
        {
          *v9++ = (8 * (v6[1] & 0xFC | (32 * (*v6 & 0xF8)))) | ((unsigned __int8)v6[2] >> 3);
          v6 += 3;
        }
      }
      else
      {
        v11 = (_BYTE *)v8[8];
        v12 = 256;
        do
        {
          --v12;
          *v11 = *v6;
          v11[1] = v6[1];
          v13 = v6[2];
          v6 += 3;
          v11[2] = v13;
          v11 += 3;
        }
        while ( v12 != 0 );
      }
      v14 = a2 + 128;
      v15 = *(unsigned __int16 *)(a2 + 66) * *(unsigned __int8 *)(a2 + 65);
      v16 = operator new(v15);
      if ( v16 == nullptr )
      {
        free(v8);
        return nullptr;
      }
      v18 = (char *)v8[7];
      v19 = 0;
      if ( a4[2] == 0 )
      {
LABEL_19:
        operator delete(v16);
        return v8;
      }
      while ( 1 )
      {
        v14 = sub_64CFC(a1, v16, v15, v14, a2, a3);
        if ( v14 == 0 )
          break;
        memcpy(v18, v16, a4[1]);
        ++v19;
        v18 += a4[1];
        if ( v19 >= a4[2] )
          goto LABEL_19;
      }
      free(v8);
      operator delete(v16);
    }
  }
  return nullptr;
}

