/* ea:   0x000180F4 */
/* name: sub_180F4 */
/* size: 68 */
int __fastcall sub_180F4(_DWORD *a1)
{
  int v2; // r3
  int result; // r0

  v2 = sub_18138();
  result = 0;
  if ( v2 != *a1 )
    return *(_DWORD *)(v2 + 20);
  return result;
}

