/* ea:   0x00063580 */
/* name: sub_63580 */
/* size: 28 */
int __fastcall sub_63580(int a1)
{
  bool v1; // zf
  int result; // r0

  v1 = *(_BYTE *)(a1 + 4) == 0;
  result = *(_DWORD *)(a1 + 8);
  if ( v1 )
    __debugbreak();
  return result;
}

