/* ea:   0x00031C18 */
/* name: sub_31C18 */
/* size: 20 */
int __fastcall sub_31C18(int a1, int a2)
{
  int result; // r0

  result = *(_DWORD *)(a1 + 44);
  *(_DWORD *)(a1 + 44) = result + a2;
  return result;
}

