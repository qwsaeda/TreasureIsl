/* ea:   0x0006A58C */
/* name: sub_6A58C */
/* size: 344 */
_DWORD *__fastcall sub_6A58C(int a1)
{
  int v2; // r5
  char v3; // r6
  _DWORD *result; // r0
  unsigned __int8 *v5; // r7
  int v6; // r8
  _BYTE *v7; // r5
  int i; // r4
  int v9; // t1
  unsigned __int16 v10; // r3
  int j; // r3
  _BYTE *v12; // r5
  int v13; // t1

  if ( a1 == 0 || *(_DWORD *)(a1 + 12) != 8 )
    __debugbreak();
  if ( a1 == 0 )
    __debugbreak();
  v2 = *(_DWORD *)(a1 + 8);
  v3 = *(_BYTE *)a1 & 0xFC;
  result = sub_62694(*(_DWORD *)(a1 + 4), v2, 24, v3);
  if ( result == nullptr )
    return nullptr;
  v5 = *(unsigned __int8 **)(a1 + 28);
  v6 = *(_DWORD *)(a1 + 24) * v2;
  v7 = (_BYTE *)result[7];
  if ( (v3 & 2) == 2 )
  {
    for ( i = *(_DWORD *)(a1 + 32); v6 != 0; v7 += 3 )
    {
      v9 = *v5++;
      --v6;
      v10 = *(_WORD *)(i + 2 * v9);
      *v7 = HIBYTE(v10) & 0xF8;
      v7[1] = (v10 >> 3) & 0xFC;
      v7[2] = 8 * v10;
    }
  }
  else
  {
    for ( j = *(_DWORD *)(a1 + 32); v6 != 0; v7 = v12 + 1 )
    {
      --v6;
      *v7 = *(_BYTE *)(3 * *v5 + j);
      v12 = v7 + 1;
      *v12++ = *(_BYTE *)(3 * *v5 + j + 1);
      v13 = *v5++;
      *v12 = *(_BYTE *)(3 * v13 + j + 2);
    }
  }
  return result;
}

