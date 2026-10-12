/* ea:   0x0005AF80 */
/* name: sub_5AF80 */
/* size: 20 */
int __fastcall sub_5AF80(int result)
{
  *(_BYTE *)(result + 22) = 1;
  *(_BYTE *)(result + 21) = 0;
  return result;
}

