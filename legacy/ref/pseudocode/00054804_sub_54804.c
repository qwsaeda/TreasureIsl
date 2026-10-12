/* ea:   0x00054804 */
/* name: sub_54804 */
/* size: 264 */
int __fastcall sub_54804(_DWORD *a1)
{
  int result; // r0
  _DWORD *v3; // r0
  _DWORD *v4; // r0

  result = a1[255];
  switch ( result )
  {
    case 1:
      result = sub_44648(a1, a1[267] + 16, a1[268] + 6);
      break;
    case 2:
      result = sub_44648(a1, a1[269] + 16, a1[270] + 6);
      break;
    case 3:
      result = sub_44648(a1, a1[271] + 16, a1[272] + 6);
      break;
    case 4:
      result = sub_44648(a1, a1[273] + 16, a1[274] + 6);
      break;
    case 5:
      result = sub_44648(a1, a1[275] + 16, a1[276] + 6);
      break;
    case 6:
      v3 = sub_445F4((int)a1, 10);
      result = sub_44648(a1, v3[3] + 21, v3[4] + 9);
      break;
    case 7:
      v4 = sub_445F4((int)a1, 20);
      result = sub_44648(a1, v4[3] + 21, v4[4] + 9);
      break;
    default:
      return result;
  }
  return result;
}

