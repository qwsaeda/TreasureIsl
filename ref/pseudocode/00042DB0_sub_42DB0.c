/* ea:   0x00042DB0 */
/* name: sub_42DB0 */
/* size: 196 */
int __fastcall sub_42DB0(int a1, _WORD *a2)
{
  return 60
       * (60
        * (24
         * (365 * (unsigned __int16)(*a2 - 2005) + 30 * (unsigned __int16)(a2[1] - 1) + (unsigned __int16)(a2[3] - 1))
         + (unsigned __int16)a2[4])
        + (unsigned __int16)a2[5])
       + (unsigned __int16)a2[6];
}

