/* ea:   0x000636E8 */
/* name: sub_636E8 */
/* size: 52 */
int __fastcall sub_636E8(int a1)
{
  int result; // r0

  result = *(_DWORD *)(a1 + 44);
  *(_DWORD *)(a1 + 44) = result + 1;
  if ( result == 0 )
    return GXSuspend();
  return result;
}

