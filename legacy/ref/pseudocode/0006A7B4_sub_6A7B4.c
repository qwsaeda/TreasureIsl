/* ea:   0x0006A7B4 */
/* name: sub_6A7B4 */
/* size: 184 */
_DWORD *__fastcall sub_6A7B4(int a1)
{
  int v2; // r4
  int v3; // r5
  _DWORD *result; // r0
  int v5; // r6
  _BYTE *v6; // r3
  _WORD *i; // r4

  if ( a1 == 0 || *(_DWORD *)(a1 + 12) != 24 )
    __debugbreak();
  if ( a1 == 0 )
    __debugbreak();
  v2 = *(_DWORD *)(a1 + 4);
  v3 = *(_DWORD *)(a1 + 8);
  result = sub_62694(v2, v3, 16, *(_BYTE *)a1 & 0xFC);
  if ( result == nullptr )
    return nullptr;
  v5 = v3 * v2;
  v6 = *(_BYTE **)(a1 + 28);
  for ( i = (_WORD *)result[7]; v5 != 0; v6 += 3 )
  {
    --v5;
    *i++ = (8 * (v6[1] & 0xFC | (32 * (*v6 & 0xF8)))) | ((unsigned __int8)v6[2] >> 3);
  }
  return result;
}

