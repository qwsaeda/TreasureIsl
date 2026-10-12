/* ea:   0x0006A6E4 */
/* name: sub_6A6E4 */
/* size: 208 */
_DWORD *__fastcall sub_6A6E4(int a1)
{
  int v2; // r5
  _DWORD *result; // r0
  unsigned __int16 *v4; // r4
  int v5; // r6
  _BYTE *i; // r5
  unsigned __int16 v7; // t1

  if ( a1 == 0 || *(_DWORD *)(a1 + 12) != 16 )
    __debugbreak();
  if ( a1 == 0 )
    __debugbreak();
  v2 = *(_DWORD *)(a1 + 8);
  result = sub_62694(*(_DWORD *)(a1 + 4), v2, 24, *(_BYTE *)a1 & 0xFC);
  if ( result == nullptr )
    return nullptr;
  v4 = *(unsigned __int16 **)(a1 + 28);
  v5 = *(_DWORD *)(a1 + 24) * v2;
  for ( i = (_BYTE *)result[7]; v5 != 0; i += 3 )
  {
    v7 = *v4++;
    --v5;
    *i = HIBYTE(v7) & 0xF8;
    i[1] = (v7 >> 3) & 0xFC;
    i[2] = 8 * v7;
  }
  return result;
}

