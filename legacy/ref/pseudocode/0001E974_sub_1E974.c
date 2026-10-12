/* ea:   0x0001E974 */
/* name: sub_1E974 */
/* size: 48 */
int __fastcall sub_1E974(int a1)
{
  int v2; // r1
  int v3; // r0
  bool v4; // zf
  int result; // r0

  if ( *(_BYTE *)(a1 + 4) == 0 )
    __debugbreak();
  v2 = *(_DWORD *)(a1 + 8);
  v3 = *(_DWORD *)(a1 + 12);
  *(_DWORD *)(a1 + 44) = v2;
  v4 = v2 == v3;
  result = 0;
  if ( !v4 )
    return v2;
  return result;
}

