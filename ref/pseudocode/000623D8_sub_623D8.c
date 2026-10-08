/* ea:   0x000623D8 */
/* name: sub_623D8 */
/* size: 80 */
int __fastcall sub_623D8(int a1)
{
  int result; // r0
  int (__fastcall ***v3)(_DWORD, int); // [sp+0h] [bp-10h] BYREF

  v3 = nullptr;
  result = sub_62428(a1 + 8, &v3);
  if ( v3 != nullptr )
    result = (**v3)(v3, 1);
  *(_BYTE *)(a1 + 4) = 0;
  return result;
}

