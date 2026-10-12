/* ea:   0x0004B1C8 */
/* name: sub_4B1C8 */
/* size: 100 */
int __fastcall sub_4B1C8(_DWORD *a1)
{
  int result; // r0
  int v3; // r3

  result = sub_443FC(a1);
  v3 = a1[255];
  switch ( v3 )
  {
    case 0:
      return sub_44648(a1, 86, 60);
    case 1:
      return sub_44648(a1, 86, 125);
    case 2:
      return sub_44648(a1, 145, 200);
    default:
      break;
  }
  return result;
}

