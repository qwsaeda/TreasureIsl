/* ea:   0x00028B00 */
/* name: sub_28B00 */
/* size: 20 */
int __fastcall sub_28B00(int a1)
{
  bool v1; // zf
  int result; // r0

  v1 = a1 == 0;
  result = *(_DWORD *)(a1 + 8);
  if ( v1 )
    __debugbreak();
  return result;
}

