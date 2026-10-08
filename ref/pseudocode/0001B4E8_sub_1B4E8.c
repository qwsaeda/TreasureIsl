/* ea:   0x0001B4E8 */
/* name: sub_1B4E8 */
/* size: 160 */
int __fastcall sub_1B4E8(_DWORD *a1)
{
  int result; // r0

  result = a1[263];
  switch ( result )
  {
    case 1:
      result = sub_44648(a1, a1[264]);
      break;
    case 2:
      result = sub_44648(a1, a1[266]);
      break;
    case 3:
      result = sub_44648(a1, a1[268]);
      break;
    case 4:
      result = sub_44648(a1, a1[270]);
      break;
    case 5:
      result = sub_44648(a1, a1[272]);
      break;
    default:
      return result;
  }
  return result;
}

