/* ea:   0x00028AEC */
/* name: sub_28AEC */
/* size: 20 */
int __fastcall sub_28AEC(int a1)
{
  bool v1; // zf
  int result; // r0

  v1 = a1 == 0;
  result = *(_DWORD *)(a1 + 4);
  if ( v1 )
    __debugbreak();
  return result;
}

